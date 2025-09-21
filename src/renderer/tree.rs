use termtree::Tree;
use netdev::Interface;

use crate::collector::sys::SysInfo;

/// Convert a string into a tree label.
fn tree_label<S: Into<String>>(s: S) -> String {
    s.into()
}

fn fmt_bps(bps: u64) -> String {
    const K: f64 = 1_000.0;
    let b = bps as f64;
    if b >= K * K * K { format!("{:.2} Gb/s", b / (K*K*K)) }
    else if b >= K * K { format!("{:.2} Mb/s", b / (K*K)) }
    else if b >= K { format!("{:.2} Kb/s", b / K) }
    else { format!("{} b/s", bps) }
}

fn fmt_flags(flags: u32) -> String {
    format!("0x{:08X}", flags)
}

/// Print the network interfaces in a tree structure.
pub fn print_interface_tree(ifaces: &[Interface]) {
    let default: bool = if ifaces.len() == 1 {
        ifaces[0].default
    } else {
        false
    };
    let host = crate::collector::sys::hostname();
    let mut root = if default {
        Tree::new(tree_label(format!("Default Interface on {}", host)))
    } else {
        Tree::new(tree_label(format!("Interfaces on {}", host)))
    };
    for iface in ifaces {
        let mut node = Tree::new(format!(
            "{}{}",
            iface.name,
            if iface.default { " (default)" } else { "" }
        ));
        
        node.push(Tree::new(format!("Index: {}", iface.index)));

        if let Some(fn_name) = &iface.friendly_name {
            node.push(Tree::new(format!("Friendly Name: {}", fn_name)));
        }
        if let Some(desc) = &iface.description {
            node.push(Tree::new(format!("Description: {}", desc)));
        }

        node.push(Tree::new(format!("Type: {:?}", iface.if_type)));
        node.push(Tree::new(format!("State: {:?}", iface.oper_state)));
        if let Some(mac) = &iface.mac_addr {
            node.push(Tree::new(format!("MAC: {}", mac)));
        }
        if let Some(mtu) = iface.mtu {
            node.push(Tree::new(format!("MTU: {}", mtu)));
        }

        if !iface.ipv4.is_empty() {
            let mut ipv4_tree = Tree::new(tree_label("IPv4"));
            for net in &iface.ipv4 {
                ipv4_tree.push(Tree::new(net.to_string()));
            }
            node.push(ipv4_tree);
        }

        if !iface.ipv6.is_empty() {
            let mut ipv6_tree = Tree::new(tree_label("IPv6"));
            for (i, net) in iface.ipv6.iter().enumerate() {
                let mut label = net.to_string();
                if let Some(scope) = iface.ipv6_scope_ids.get(i) {
                    label.push_str(&format!(" (scope_id={})", scope));
                }
                ipv6_tree.push(Tree::new(label));
            }
            node.push(ipv6_tree);
        }

        if !iface.dns_servers.is_empty() {
            let mut dns_tree = Tree::new(tree_label("DNS"));
            for dns in &iface.dns_servers {
                dns_tree.push(Tree::new(dns.to_string()));
            }
            node.push(dns_tree);
        }

        if let Some(gw) = &iface.gateway {
            let mut gw_node = Tree::new(tree_label("Gateway"));
            // GW MAC
            gw_node.push(Tree::new(format!("MAC: {}", gw.mac_addr)));
            // GW IPv4/IPv6
            if !gw.ipv4.is_empty() {
                let mut gw_tree = Tree::new(tree_label("IPv4"));
                for ip in &gw.ipv4 {
                    gw_tree.push(Tree::new(ip.to_string()));
                }
                gw_node.push(gw_tree);
            }
            if !gw.ipv6.is_empty() {
                let mut gw_tree = Tree::new(tree_label("IPv6"));
                for ip in &gw.ipv6 {
                    gw_tree.push(Tree::new(ip.to_string()));
                }
                gw_node.push(gw_tree);
            }
            node.push(gw_node);
        }

        root.push(node);
    }
    println!("{}", root);
}

/// Print detailed information of a single interface in a tree structure.
pub fn print_interface_detail_tree(iface: &Interface) {
    let host = crate::collector::sys::hostname();
    let title = format!(
        "{}{} on {}",
        iface.name,
        if iface.default { " (default)" } else { "" },
        host
    );
    let mut root = Tree::new(tree_label(title));

    // flat fields (no General section)
    root.push(Tree::new(format!("Index: {}", iface.index)));

    if let Some(fn_name) = &iface.friendly_name {
        root.push(Tree::new(format!("Friendly Name: {}", fn_name)));
    }
    if let Some(desc) = &iface.description {
        root.push(Tree::new(format!("Description: {}", desc)));
    }

    root.push(Tree::new(format!("Type: {:?}", iface.if_type)));
    root.push(Tree::new(format!("State: {:?}", iface.oper_state)));

    if let Some(mac) = &iface.mac_addr {
        root.push(Tree::new(format!("MAC: {}", mac)));
    }
    if let Some(mtu) = iface.mtu {
        root.push(Tree::new(format!("MTU: {}", mtu)));
    }

    // link speeds (humanized bps)
    if iface.transmit_speed.is_some() || iface.receive_speed.is_some() {
        let mut speed = Tree::new(tree_label("Link Speed"));
        if let Some(tx) = iface.transmit_speed { speed.push(Tree::new(format!("TX: {}", fmt_bps(tx)))); }
        if let Some(rx) = iface.receive_speed { speed.push(Tree::new(format!("RX: {}", fmt_bps(rx)))); }
        root.push(speed);
    }

    // flags
    root.push(Tree::new(format!("Flags: {}", fmt_flags(iface.flags))));

    // ---- Addresses ----
    if !iface.ipv4.is_empty() {
        let mut ipv4_tree = Tree::new(tree_label("IPv4"));
        for net in &iface.ipv4 { ipv4_tree.push(Tree::new(net.to_string())); }
        root.push(ipv4_tree);
    }

    if !iface.ipv6.is_empty() {
        let mut ipv6_tree = Tree::new(tree_label("IPv6"));
        for (i, net) in iface.ipv6.iter().enumerate() {
            let mut label = net.to_string();
            if let Some(scope) = iface.ipv6_scope_ids.get(i) { label.push_str(&format!(" (scope_id={})", scope)); }
            ipv6_tree.push(Tree::new(label));
        }
        root.push(ipv6_tree);
    }

    // ---- DNS ----
    if !iface.dns_servers.is_empty() {
        let mut dns_tree = Tree::new(tree_label("DNS"));
        for dns in &iface.dns_servers { dns_tree.push(Tree::new(dns.to_string())); }
        root.push(dns_tree);
    }

    // ---- Gateway ----
    if let Some(gw) = &iface.gateway {
        let mut gw_node = Tree::new(tree_label("Gateway"));
        gw_node.push(Tree::new(format!("MAC: {}", gw.mac_addr)));
        if !gw.ipv4.is_empty() {
            let mut gw4 = Tree::new(tree_label("IPv4"));
            for ip in &gw.ipv4 { gw4.push(Tree::new(ip.to_string())); }
            gw_node.push(gw4);
        }
        if !gw.ipv6.is_empty() {
            let mut gw6 = Tree::new(tree_label("IPv6"));
            for ip in &gw.ipv6 { gw6.push(Tree::new(ip.to_string())); }
            gw_node.push(gw6);
        }
        root.push(gw_node);
    }

    // ---- Statistics (snapshot) ----
    if let Some(st) = &iface.stats {
        let mut stats_node = Tree::new(tree_label("Statistics (snapshot)"));
        stats_node.push(Tree::new(format!("RX bytes: {}", st.rx_bytes)));
        stats_node.push(Tree::new(format!("TX bytes: {}", st.tx_bytes)));
        root.push(stats_node);
    }

    println!("{}", root);
}

pub fn print_system_with_default_iface(sys: &SysInfo, default_iface: Option<Interface>) {
    let mut root = Tree::new(tree_label(format!("System Information on {}", sys.hostname)));

    // ---- System ----
    let mut sys_node = Tree::new(tree_label("System"));
    sys_node.push(Tree::new(tree_label(format!("OS Type: {}", sys.os_type))));
    sys_node.push(Tree::new(tree_label(format!("Version: {}", sys.os_version))));
    sys_node.push(Tree::new(tree_label(format!("Edition: {}", sys.edition))));
    sys_node.push(Tree::new(tree_label(format!("Codename: {}", sys.codename))));
    sys_node.push(Tree::new(tree_label(format!("Bitness: {}", sys.bitness))));
    sys_node.push(Tree::new(tree_label(format!("Architecture: {}", sys.architecture))));
    root.push(sys_node);

    // ---- Default Interface (optional) ----
    if let Some(iface) = default_iface {
        let mut if_node = Tree::new(tree_label(format!("Default Interface: {}", iface.name)));

        if let Some(fn_name) = &iface.friendly_name {
            if_node.push(Tree::new(tree_label(format!("Friendly Name: {}", fn_name))));
        }
        if let Some(desc) = &iface.description {
            if_node.push(Tree::new(tree_label(format!("Description: {}", desc))));
        }

        if_node.push(Tree::new(tree_label(format!("Index: {}", iface.index))));
        if_node.push(Tree::new(tree_label(format!("Type: {:?}", iface.if_type))));
        if_node.push(Tree::new(tree_label(format!("State: {:?}", iface.oper_state))));
        if let Some(mac) = &iface.mac_addr {
            if_node.push(Tree::new(tree_label(format!("MAC: {}", mac))));
        }
        if let Some(mtu) = iface.mtu {
            if_node.push(Tree::new(tree_label(format!("MTU: {}", mtu))));
        }

        // Speeds
        if iface.transmit_speed.is_some() || iface.receive_speed.is_some() {
            let mut speed = Tree::new(tree_label("Link Speed"));
            if let Some(tx) = iface.transmit_speed {
                speed.push(Tree::new(tree_label(format!("TX: {}", fmt_bps(tx)))));
            }
            if let Some(rx) = iface.receive_speed {
                speed.push(Tree::new(tree_label(format!("RX: {}", fmt_bps(rx)))));
            }
            if_node.push(speed);
        }

        // IPv4
        if !iface.ipv4.is_empty() {
            let mut ipv4_node = Tree::new(tree_label("IPv4"));
            for n in &iface.ipv4 {
                ipv4_node.push(Tree::new(tree_label(n.to_string())));
            }
            if_node.push(ipv4_node);
        }
        // IPv6 with scope ID
        if !iface.ipv6.is_empty() {
            let mut ipv6_node = Tree::new(tree_label("IPv6"));
            for (i, n) in iface.ipv6.iter().enumerate() {
                let mut label = n.to_string();
                if let Some(sc) = iface.ipv6_scope_ids.get(i) {
                    label.push_str(&format!(" (scope_id={})", sc));
                }
                ipv6_node.push(Tree::new(tree_label(label)));
            }
            if_node.push(ipv6_node);
        }

        // DNS
        if !iface.dns_servers.is_empty() {
            let mut dns = Tree::new(tree_label("DNS"));
            for s in &iface.dns_servers {
                dns.push(Tree::new(tree_label(s.to_string())));
            }
            if_node.push(dns);
        }

        // Gateway (IP + MAC)
        if let Some(gw) = &iface.gateway {
            let mut gw_node = Tree::new(tree_label("Gateway"));
            gw_node.push(Tree::new(tree_label(format!("MAC: {}", gw.mac_addr))));
            if !gw.ipv4.is_empty() {
                let mut gw4 = Tree::new(tree_label("IPv4"));
                for ip in &gw.ipv4 {
                    gw4.push(Tree::new(tree_label(ip.to_string())));
                }
                gw_node.push(gw4);
            }
            if !gw.ipv6.is_empty() {
                let mut gw6 = Tree::new(tree_label("IPv6"));
                for ip in &gw.ipv6 {
                    gw6.push(Tree::new(tree_label(ip.to_string())));
                }
                gw_node.push(gw6);
            }
            if_node.push(gw_node);
        }

        root.push(if_node);
    } else {
        root.push(Tree::new(tree_label("Default Interface: (not found)")));
    }

    println!("{}", root);
}
