use crate::cli::Cli;
use crate::cli::IfacesArgs;
use crate::db::oui::is_oui_db_initialized;
use crate::net;
use crate::renderer;
use crate::renderer::table::make_table;
use crate::renderer::tree::tree_label;
use mac_addr::MacAddr;
use netdev::Interface;
use netdev::interface::state::OperState;
use termtree::Tree;

pub fn list_interfaces(_cli: &Cli, args: &IfacesArgs) {
    let mut interfaces: Vec<Interface> = net::iface::get_all_interfaces();

    // Apply filters
    if let Some(name_like) = &args.name_like {
        interfaces.retain(|iface| iface.name.contains(name_like));
    }
    if args.up {
        interfaces.retain(|iface| iface.oper_state == OperState::Up);
    }
    if args.down {
        interfaces.retain(|iface| iface.oper_state == OperState::Down);
    }
    if args.phy {
        interfaces.retain(|iface| iface.is_physical());
    }
    if args.virt {
        interfaces.retain(|iface| !iface.is_physical());
    }
    if args.ipv4 {
        interfaces.retain(|iface| !iface.ipv4.is_empty());
    }
    if args.ipv6 {
        interfaces.retain(|iface| !iface.ipv6.is_empty());
    }

    if args.export {
        crate::fs::export(
            args.format,
            args.output.as_deref(),
            &interfaces,
        ).unwrap_or_else(|e| {
            tracing::error!("Export failed: {}", e);
        });
    }else{
        // Render output
        match args.format {
            crate::cli::OutputFormat::Tree => print_interface_tree(&interfaces),
            crate::cli::OutputFormat::Json => renderer::json::print_interface_json(&interfaces),
            crate::cli::OutputFormat::Yaml => renderer::yaml::print_interface_yaml(&interfaces),
            crate::cli::OutputFormat::Table => print_interface_table(&interfaces),
        }
    }
}

/// Print the network interfaces in a tree structure.
fn print_interface_tree(ifaces: &[Interface]) {
    let default: bool = if ifaces.len() == 1 {
        ifaces[0].default
    } else {
        false
    };
    let host = crate::net::sys::hostname();
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

            if is_oui_db_initialized() && *mac != MacAddr::zero() {
                let oui_db = crate::db::oui::oui_db();
                if let Some(vendor) = oui_db.lookup_mac(mac) {
                    let vendor_name = vendor.vendor_detail.as_deref().unwrap_or(&vendor.vendor);
                    node.push(Tree::new(format!("Vendor: {}", vendor_name)));
                }
            }
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

        if iface.default {
            let vpn_heuristic = crate::net::iface::detect_vpn_like(&iface);
            if vpn_heuristic.is_vpn_like {
                let mut heuristic_node = Tree::new(tree_label("Heuristic"));
                heuristic_node.push(Tree::new(format!(
                    "VPN-like: {}",
                    vpn_heuristic.is_vpn_like
                )));
                node.push(heuristic_node);
            }
        }

        root.push(node);
    }
    println!("{}", root);
}

fn print_interface_table(ifs: &[Interface]) {
    let mut table = make_table(&["INDEX", "NAME", "TYPE", "STATE", "MAC", "IPv4", "IPv6", "MTU"]);
    for iface in ifs {
        let ipv4 = iface.ipv4
            .iter()
            .map(|n| n.addr().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let ipv6 = iface.ipv6
            .iter()
            .map(|n| n.addr().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        table.add_row(vec![
            iface.index.to_string(),
            iface.name.clone(),
            iface.if_type.name(),
            iface.oper_state.to_string(),
            iface.mac_addr
                .map(|m| m.to_string())
                .unwrap_or_else(|| "-".into()),
            ipv4,
            ipv6,
            iface.mtu
                .map(|m| m.to_string())
                .unwrap_or_else(|| "-".into()),
        ]);
    }

    println!("{table}");
}
