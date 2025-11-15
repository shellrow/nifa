use anyhow::Result;
use netdev::Interface;
use termtree::Tree;
use mac_addr::MacAddr;
use crate::cli::Cli;
use crate::cli::IfaceArgs;
use crate::db::oui::is_oui_db_initialized;
use crate::net;
use crate::renderer;
use crate::renderer::fmt_bps;
use crate::renderer::fmt_flags;
use crate::renderer::tree::tree_label;

/// Default action with no subcommand
pub fn show_default_interface(_cli: &Cli) -> Result<()> {
    let iface: Interface = net::iface::get_default_interface()
        .ok_or_else(|| anyhow::anyhow!("No default interface found"))?;
    // Render output
    print_default_interface_tree(&iface);

    // Print note about nifa help
    println!();
    println!("Tip: Run 'nifa --help' to see available commands and options.");
    Ok(())
}

/// Show specified interface details
pub fn show_interface(_cli: &Cli, args: &IfaceArgs) -> Result<()> {
    if args.vendor {
        crate::db::oui::init_oui_db()?;
    }
    match net::iface::get_interface_by_name(&args.iface) {
        Some(iface) => {
            match args.export {
                Some(export_format) => {
                    // Export to file in specified format
                    crate::fs::export(
                        export_format,
                        args.output.as_deref(),
                        &iface,
                    )?;
                    return Ok(());
                }
                None => {
                    // Render output
                    match args.format {
                        crate::cli::OutputFormat::Tree => print_interface_detail_tree(&iface),
                        crate::cli::OutputFormat::Json => renderer::json::print_interface_json(&[iface]),
                        crate::cli::OutputFormat::Yaml => renderer::yaml::print_interface_yaml(&[iface]),
                        _ => {
                            tracing::error!(
                                "Unsupported format for show interface: {:?}",
                                args.format
                            );
                        }
                    }
                }
            }
        }
        None => {
            tracing::error!("Interface '{}' not found", args.iface);
        }
    }
    Ok(())
}

/// Print detailed information of a single interface in a tree structure.
fn print_default_interface_tree(iface: &Interface) {
    let host = crate::net::sys::hostname();
    let title = format!(
        "Default Network Interface on {}",
        host
    );
    let mut root = Tree::new(tree_label(title));

    // flat fields (no General section)
    root.push(Tree::new(format!("Index: {}", iface.index)));
    root.push(Tree::new(format!("Name: {}", iface.name)));

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

        if is_oui_db_initialized() && *mac != MacAddr::zero() {
            let oui_db = crate::db::oui::oui_db();
            if let Some(vendor) = oui_db.lookup_mac(mac) {
                let vendor_name = vendor.vendor_detail.as_deref().unwrap_or(&vendor.vendor);
                root.push(Tree::new(format!("Vendor: {}", vendor_name)));
            }
        }
    }

    if let Some(mtu) = iface.mtu {
        root.push(Tree::new(format!("MTU: {}", mtu)));
    }

    // link speeds (humanized bps)
    if iface.transmit_speed.is_some() || iface.receive_speed.is_some() {
        let mut speed = Tree::new(tree_label("Link Speed"));
        if let Some(tx) = iface.transmit_speed {
            speed.push(Tree::new(format!("TX: {}", fmt_bps(tx))));
        }
        if let Some(rx) = iface.receive_speed {
            speed.push(Tree::new(format!("RX: {}", fmt_bps(rx))));
        }
        root.push(speed);
    }

    // flags
    root.push(Tree::new(format!("Flags: {}", fmt_flags(iface.flags))));

    // ---- Addresses ----
    if !iface.ipv4.is_empty() {
        let mut ipv4_tree = Tree::new(tree_label("IPv4"));
        for net in &iface.ipv4 {
            ipv4_tree.push(Tree::new(net.to_string()));
        }
        root.push(ipv4_tree);
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
        root.push(ipv6_tree);
    }

    // ---- DNS ----
    if !iface.dns_servers.is_empty() {
        let mut dns_tree = Tree::new(tree_label("DNS"));
        for dns in &iface.dns_servers {
            dns_tree.push(Tree::new(dns.to_string()));
        }
        root.push(dns_tree);
    }

    // ---- Gateway ----
    if let Some(gw) = &iface.gateway {
        let mut gw_node = Tree::new(tree_label("Gateway"));
        gw_node.push(Tree::new(format!("MAC: {}", gw.mac_addr)));
        if !gw.ipv4.is_empty() {
            let mut gw4 = Tree::new(tree_label("IPv4"));
            for ip in &gw.ipv4 {
                gw4.push(Tree::new(ip.to_string()));
            }
            gw_node.push(gw4);
        }
        if !gw.ipv6.is_empty() {
            let mut gw6 = Tree::new(tree_label("IPv6"));
            for ip in &gw.ipv6 {
                gw6.push(Tree::new(ip.to_string()));
            }
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

    let vpn_heuristic = crate::net::iface::detect_vpn_like(&iface);
    if vpn_heuristic.is_vpn_like {
        let mut heuristic_node = Tree::new(tree_label("Heuristic"));
        heuristic_node.push(Tree::new(format!(
            "VPN-like: {}",
            vpn_heuristic.is_vpn_like
        )));
        root.push(heuristic_node);
    }

    println!("{}", root);
}

/// Print detailed information of a single interface in a tree structure.
fn print_interface_detail_tree(iface: &Interface) {
    let host = crate::net::sys::hostname();
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

        if is_oui_db_initialized() && *mac != MacAddr::zero() {
            let oui_db = crate::db::oui::oui_db();
            if let Some(vendor) = oui_db.lookup_mac(mac) {
                let vendor_name = vendor.vendor_detail.as_deref().unwrap_or(&vendor.vendor);
                root.push(Tree::new(format!("Vendor: {}", vendor_name)));
            }
        }
    }

    if let Some(mtu) = iface.mtu {
        root.push(Tree::new(format!("MTU: {}", mtu)));
    }

    // link speeds (humanized bps)
    if iface.transmit_speed.is_some() || iface.receive_speed.is_some() {
        let mut speed = Tree::new(tree_label("Link Speed"));
        if let Some(tx) = iface.transmit_speed {
            speed.push(Tree::new(format!("TX: {}", fmt_bps(tx))));
        }
        if let Some(rx) = iface.receive_speed {
            speed.push(Tree::new(format!("RX: {}", fmt_bps(rx))));
        }
        root.push(speed);
    }

    // flags
    root.push(Tree::new(format!("Flags: {}", fmt_flags(iface.flags))));

    // ---- Addresses ----
    if !iface.ipv4.is_empty() {
        let mut ipv4_tree = Tree::new(tree_label("IPv4"));
        for net in &iface.ipv4 {
            ipv4_tree.push(Tree::new(net.to_string()));
        }
        root.push(ipv4_tree);
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
        root.push(ipv6_tree);
    }

    // ---- DNS ----
    if !iface.dns_servers.is_empty() {
        let mut dns_tree = Tree::new(tree_label("DNS"));
        for dns in &iface.dns_servers {
            dns_tree.push(Tree::new(dns.to_string()));
        }
        root.push(dns_tree);
    }

    // ---- Gateway ----
    if let Some(gw) = &iface.gateway {
        let mut gw_node = Tree::new(tree_label("Gateway"));
        gw_node.push(Tree::new(format!("MAC: {}", gw.mac_addr)));
        if !gw.ipv4.is_empty() {
            let mut gw4 = Tree::new(tree_label("IPv4"));
            for ip in &gw.ipv4 {
                gw4.push(Tree::new(ip.to_string()));
            }
            gw_node.push(gw4);
        }
        if !gw.ipv6.is_empty() {
            let mut gw6 = Tree::new(tree_label("IPv6"));
            for ip in &gw.ipv6 {
                gw6.push(Tree::new(ip.to_string()));
            }
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

    let vpn_heuristic = crate::net::iface::detect_vpn_like(&iface);
    if vpn_heuristic.is_vpn_like {
        let mut heuristic_node = Tree::new(tree_label("Heuristic"));
        heuristic_node.push(Tree::new(format!(
            "VPN-like: {}",
            vpn_heuristic.is_vpn_like
        )));
        root.push(heuristic_node);
    }

    println!("{}", root);
}
