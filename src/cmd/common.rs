use netdev::Interface;

use crate::model::{IfDetail, IfSummary, IpAddrEntry};
use crate::renderer::{fmt_bps, fmt_flags};

pub fn vendor_name_for(mac: &Option<mac_addr::MacAddr>) -> Option<String> {
    let mac = mac.as_ref()?;
    if !crate::db::oui::is_oui_db_initialized() || *mac == mac_addr::MacAddr::zero() {
        return None;
    }
    let oui_db = crate::db::oui::oui_db();
    oui_db
        .lookup_mac(mac)
        .map(|v| v.vendor_detail.as_deref().unwrap_or(&v.vendor).to_string())
}

pub fn interface_to_summary(iface: &Interface, with_vendor: bool) -> IfSummary {
    let (gateway_mac, gateway_ipv4, gateway_ipv6) = if let Some(gw) = &iface.gateway {
        let mac = if gw.mac_addr == mac_addr::MacAddr::zero() {
            None
        } else {
            Some(gw.mac_addr.to_string())
        };
        let ipv4: Vec<String> = gw
            .ipv4
            .iter()
            .filter(|ip| !ip.is_unspecified())
            .map(|ip| ip.to_string())
            .collect();
        let ipv6: Vec<String> = gw
            .ipv6
            .iter()
            .filter(|ip| {
                !ip.is_unspecified() && **ip != std::net::Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0)
            })
            .map(|ip| ip.to_string())
            .collect();
        if mac.is_some() || !ipv4.is_empty() || !ipv6.is_empty() {
            (mac, ipv4, ipv6)
        } else {
            (None, Vec::new(), Vec::new())
        }
    } else {
        (None, Vec::new(), Vec::new())
    };

    IfSummary {
        index: iface.index,
        name: iface.name.clone(),
        if_type: iface.if_type.name().to_string(),
        state: iface.oper_state.to_string(),
        is_default: iface.default,
        mac: iface.mac_addr.map(|m| m.to_string()),
        mtu: iface.mtu,
        ipv4_addrs: iface.ipv4.iter().map(|n| n.to_string()).collect(),
        ipv6_addrs: iface.ipv6.iter().map(|n| n.to_string()).collect(),
        gateway_mac,
        gateway_ipv4,
        gateway_ipv6,
        vendor: if with_vendor {
            vendor_name_for(&iface.mac_addr)
        } else {
            None
        },
    }
}

pub fn interface_to_detail(iface: &Interface, with_vendor: bool) -> IfDetail {
    let gateway_mac = iface.gateway.as_ref().map(|g| g.mac_addr.to_string());
    let gateway_ipv4 = iface
        .gateway
        .as_ref()
        .map(|g| g.ipv4.iter().map(|ip| ip.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    let gateway_ipv6 = iface
        .gateway
        .as_ref()
        .map(|g| g.ipv6.iter().map(|ip| ip.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    let vpn_heuristic = crate::net::iface::detect_vpn_like(iface);

    IfDetail {
        summary: interface_to_summary(iface, with_vendor),
        friendly_name: iface.friendly_name.clone(),
        description: iface.description.clone(),
        flags: fmt_flags(iface.flags),
        tx_speed: iface.transmit_speed.map(fmt_bps),
        rx_speed: iface.receive_speed.map(fmt_bps),
        ipv4: iface.ipv4.iter().map(|n| n.to_string()).collect(),
        ipv6: iface.ipv6.iter().map(|n| n.to_string()).collect(),
        ipv6_scoped: iface
            .ipv6
            .iter()
            .enumerate()
            .map(|(idx, n)| {
                if let Some(scope) = iface.ipv6_scope_ids.get(idx) {
                    format!("{} (scope_id={scope})", n)
                } else {
                    n.to_string()
                }
            })
            .collect(),
        dns_servers: iface.dns_servers.iter().map(|d| d.to_string()).collect(),
        gateway_ipv4,
        gateway_ipv6,
        gateway_mac,
        stats_rx_bytes: iface.stats.as_ref().map(|s| s.rx_bytes),
        stats_tx_bytes: iface.stats.as_ref().map(|s| s.tx_bytes),
        vpn_like: vpn_heuristic.is_vpn_like,
    }
}

pub fn interface_to_ip_entries(iface: &Interface) -> Vec<IpAddrEntry> {
    let mut out = Vec::with_capacity(iface.ipv4.len() + iface.ipv6.len());
    for net in &iface.ipv4 {
        out.push(IpAddrEntry {
            iface: iface.name.clone(),
            if_type: iface.if_type.name().to_string(),
            family: "ipv4".to_string(),
            address: net.addr().to_string(),
            prefix_len: net.prefix_len(),
            scope_id: None,
        });
    }
    for (idx, net) in iface.ipv6.iter().enumerate() {
        out.push(IpAddrEntry {
            iface: iface.name.clone(),
            if_type: iface.if_type.name().to_string(),
            family: "ipv6".to_string(),
            address: net.addr().to_string(),
            prefix_len: net.prefix_len(),
            scope_id: iface.ipv6_scope_ids.get(idx).copied(),
        });
    }
    out
}
