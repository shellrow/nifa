use netdev::Interface;
use netdev::interface::InterfaceType;

/// Common patterns that indicate a VPN/tunnel adapter
const VPN_NAME_PATTERNS: &[&str] = &[
    "tun",
    "tap",
    "wg",
    "tailscale",
    "zerotier",
    "zt",
    "openvpn",
    "ovpn",
    "ipsec",
    "utun",
    "vpn",
    "adapter",
    "wan miniport",
    "nord",
    "expressvpn",
];

pub fn collect_all_interfaces() -> Vec<Interface> {
    netdev::get_interfaces()
}

pub fn get_default_interface() -> Option<Interface> {
    match netdev::get_default_interface() {
        Ok(iface) => Some(iface),
        Err(_) => None,
    }
}

pub fn get_interface_by_name(name: &str) -> Option<Interface> {
    let interfaces = netdev::get_interfaces();
    for iface in interfaces {
        if iface.name == name {
            return Some(iface);
        }
    }
    None
}

#[derive(Debug)]
pub struct VpnHeuristic {
    pub is_vpn_like: bool,
    #[allow(dead_code)]
    pub score: i32,
    #[allow(dead_code)]
    pub signals: Vec<String>,
}

/// Check if the given interface looks like a VPN interface using simple heuristics.
pub fn detect_vpn_like(default_if: &Interface) -> VpnHeuristic {
    let mut score = 0;
    let mut sig = Vec::new();

    // Check InterfaceType
    match default_if.if_type {
        InterfaceType::Tunnel | InterfaceType::Ppp | InterfaceType::ProprietaryVirtual => {
            score += 4;
            sig.push(format!("type={:?}", default_if.if_type));
        }
        _ => {}
    }

    // Check name patterns
    let name = default_if.name.to_lowercase();
    if VPN_NAME_PATTERNS.iter().any(|p| name.contains(p)) {
        score += 3;
        sig.push(format!("name={}", default_if.name));
    }

    // Check friendly_name patterns
    if let Some(fname) = &default_if.friendly_name {
        let fname_lower = fname.to_lowercase();
        if VPN_NAME_PATTERNS.iter().any(|p| fname_lower.contains(p)) {
            score += 3;
            sig.push(format!("friendly_name={}", fname));
        }
    }

    // Check MTU
    if let Some(mtu) = default_if.mtu {
        if mtu < 1500 {
            // Likely VPN MTU
            score += if (1410..=1460).contains(&mtu) { 2 } else { 1 };
            sig.push(format!("mtu={}", mtu));
        }
    }

    // Check if IPv4 is 10/8 or 100.64/10
    let v4_inner_like = default_if.ipv4.iter().any(|n| {
        let ip = n.addr();
        let oct = ip.octets();
        oct[0] == 10 || (oct[0] == 100 && (oct[1] & 0b1100_0000) == 0b0100_0000) // 100.64.0.0/10
    });
    if v4_inner_like {
        score += 2;
        sig.push("ipv4=private(10/8 or 100.64/10)".into());
    }

    // Check if DNS is 100.64/10
    let dns_any_100_64 = default_if.dns_servers.iter().any(|ip| {
        if let std::net::IpAddr::V4(v4) = ip {
            let o = v4.octets();
            o[0] == 100 && (o[1] & 0b1100_0000) == 0b0100_0000
        } else {
            false
        }
    });
    if dns_any_100_64 {
        score += 1;
        sig.push("dns=100.64.0.0/10".into());
    }

    // Check if the type is clearly not physical
    match default_if.if_type {
        InterfaceType::Ethernet
        | InterfaceType::Wireless80211
        | InterfaceType::GigabitEthernet
        | InterfaceType::FastEthernetT
        | InterfaceType::FastEthernetFx => {}
        _ => {
            score += 1;
            sig.push(format!("type-other={:?}", default_if.if_type));
        }
    }

    let is_vpn_like = score >= 5;
    VpnHeuristic {
        is_vpn_like,
        score,
        signals: sig,
    }
}
