use anyhow::Result;
use netdev::interface::state::OperState;
use termtree::Tree;

use crate::cli::{IfArgs, OutputFormat, ShowAction};
use crate::model::{IfDetail, IfSummary};
use crate::renderer::fmt_flags;

pub fn run(args: &IfArgs) -> Result<()> {
    if args.vendor {
        crate::db::oui::init_oui_db()?;
    }

    match &args.action {
        Some(ShowAction::Show { target }) => {
            let iface = if let Some(primary) = crate::net::iface::get_primary_interface() {
                if primary.name == *target {
                    primary
                } else {
                    crate::net::iface::get_interface_by_name(target)
                        .ok_or_else(|| anyhow::anyhow!("Interface '{target}' not found"))?
                }
            } else {
                crate::net::iface::get_interface_by_name(target)
                    .ok_or_else(|| anyhow::anyhow!("Interface '{target}' not found"))?
            };
            let detail: IfDetail = super::common::interface_to_detail(&iface, args.vendor);
            let mut out = args.out.clone();
            if matches!(out.format, OutputFormat::Auto) {
                out.format = OutputFormat::Tree;
            }
            crate::renderer::render_if_detail(&detail, &out)
        }
        None => {
            let mut interfaces = crate::net::iface::get_all_interfaces();
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
            interfaces.sort_by(|a, b| a.name.cmp(&b.name));

            let summaries: Vec<IfSummary> = interfaces
                .iter()
                .map(|iface| super::common::interface_to_summary(iface, args.vendor))
                .collect();
            crate::renderer::render_if_summaries(&summaries, &args.out)
        }
    }
}

pub fn run_default_interface() -> Result<()> {
    let iface = crate::net::iface::get_primary_interface()
        .ok_or_else(|| anyhow::anyhow!("No network interface found"))?;
    let host = crate::net::sys::hostname();

    let mut root = Tree::new(format!("Default Network Interface on {}", host));
    root.push(Tree::new(format!("Index: {}", iface.index)));
    root.push(Tree::new(format!("Name: {}", iface.name)));
    if let Some(name) = &iface.friendly_name {
        root.push(Tree::new(format!("Friendly Name: {}", name)));
    }
    root.push(Tree::new(format!("Type: {}", iface.if_type.name())));
    root.push(Tree::new(format!("State: {}", iface.oper_state)));
    if let Some(mac) = &iface.mac_addr {
        root.push(Tree::new(format!("MAC: {}", mac)));
    }
    if let Some(mtu) = iface.mtu {
        root.push(Tree::new(format!("MTU: {}", mtu)));
    }
    root.push(Tree::new(format!("Flags: {}", fmt_flags(iface.flags))));

    let mut ipv4 = Tree::new("IPv4".to_string());
    if iface.ipv4.is_empty() {
        ipv4.push(Tree::new("(none)".to_string()));
    } else {
        for net in &iface.ipv4 {
            ipv4.push(Tree::new(net.to_string()));
        }
    }
    root.push(ipv4);

    let mut ipv6 = Tree::new("IPv6".to_string());
    if iface.ipv6.is_empty() {
        ipv6.push(Tree::new("(none)".to_string()));
    } else {
        for (idx, net) in iface.ipv6.iter().enumerate() {
            let mut label = net.to_string();
            if let Some(scope) = iface.ipv6_scope_ids.get(idx) {
                label.push_str(&format!(" (scope_id={scope})"));
            }
            ipv6.push(Tree::new(label));
        }
    }
    root.push(ipv6);

    let mut dns = Tree::new("DNS".to_string());
    if iface.dns_servers.is_empty() {
        dns.push(Tree::new("(none)".to_string()));
    } else {
        for server in &iface.dns_servers {
            dns.push(Tree::new(server.to_string()));
        }
    }
    root.push(dns);

    if let Some(gw) = &iface.gateway {
        let mut gateway = Tree::new("Gateway".to_string());
        gateway.push(Tree::new(format!("MAC: {}", gw.mac_addr)));
        let mut gw4 = Tree::new("IPv4".to_string());
        if gw.ipv4.is_empty() {
            gw4.push(Tree::new("(none)".to_string()));
        } else {
            for ip in &gw.ipv4 {
                gw4.push(Tree::new(ip.to_string()));
            }
        }
        gateway.push(gw4);
        let mut gw6 = Tree::new("IPv6".to_string());
        if gw.ipv6.is_empty() {
            gw6.push(Tree::new("(none)".to_string()));
        } else {
            for ip in &gw.ipv6 {
                gw6.push(Tree::new(ip.to_string()));
            }
        }
        gateway.push(gw6);
        root.push(gateway);
    }

    if let Some(st) = &iface.stats {
        let mut stats = Tree::new("Statistics (snapshot)".to_string());
        stats.push(Tree::new(format!("RX bytes: {}", st.rx_bytes)));
        stats.push(Tree::new(format!("TX bytes: {}", st.tx_bytes)));
        root.push(stats);
    }

    let vpn = crate::net::iface::detect_vpn_like(&iface);
    root.push(Tree::new(format!("VPN-like: {}", vpn.is_vpn_like)));

    println!("{root}");
    println!();
    println!("Tip: Run 'nifa --help' to see available commands and options.");
    Ok(())
}
