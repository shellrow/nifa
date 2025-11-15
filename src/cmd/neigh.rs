use std::collections::HashMap;
use std::net::IpAddr;

use anyhow::Result;
use mac_addr::MacAddr;
use crate::cli::{Cli, OutputFormat, NeighArgs};
use crate::db::oui::is_oui_db_initialized;
use crate::net::neigh;
use crate::renderer::table::make_table;
use termtree::Tree;
use crate::renderer::tree::tree_label;
use netdev::NetworkDevice;

pub fn show_neigh(_cli: &Cli, args: &NeighArgs) -> Result<()> {
    let table: HashMap<IpAddr, MacAddr> = neigh::get_neighbor_table()?;
    match args.export {
        Some(export_format) => {
            let devices = map_to_devices(table);
            crate::fs::export(
                export_format,
                args.output.as_deref(),
                &devices,
            )?;
            return Ok(());
        }
        None => {
            match args.format {
                OutputFormat::Tree => print_neigh_tree(&table),
                OutputFormat::Table => print_neigh_table(&table),
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&table)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&table)?),
            }
        }
    }
    Ok(())
}

fn map_to_devices(map: HashMap<IpAddr, MacAddr>) -> Vec<NetworkDevice> {
    map.into_iter().map(|(ip, mac)| {
        let mut device = NetworkDevice::new();
        device.mac_addr = mac;
        match ip {
            IpAddr::V4(v4) => device.ipv4.push(v4),
            IpAddr::V6(v6) => device.ipv6.push(v6),
        }
        device
    }).collect()
}

fn print_neigh_tree(table: &std::collections::HashMap<IpAddr, MacAddr>) {
    let iface = netdev::get_default_interface().unwrap();
    let self_ips: Vec<IpAddr> = iface.ip_addrs();
    let host = crate::net::sys::hostname();
    let mut root = Tree::new(tree_label(format!("Neighbors (ARP/NDP) on {}", host)));

    let mut v4 = Tree::new(tree_label("IPv4"));
    let mut v6 = Tree::new(tree_label("IPv6"));

    let mut keys: Vec<_> = table.keys().cloned().collect();
    keys.sort_by(|a,b| a.to_string().cmp(&b.to_string()));

    for ip in keys {
        let mac = table.get(&ip).unwrap();
        let mut ip_node = Tree::new(tree_label(ip.to_string()));
        ip_node.push(Tree::new(format!("MAC: {}", mac)));
        // Vendor lookup
        if is_oui_db_initialized() && *mac != MacAddr::zero() && !mac.is_broadcast() {
            let oui_db = crate::db::oui::oui_db();
            if let Some(vendor) = oui_db.lookup_mac(mac) {
                let vendor_name = vendor.vendor_detail.as_deref().unwrap_or(&vendor.vendor);
                ip_node.push(Tree::new(format!("Vendor: {}", vendor_name)));
            }
        }

        // Classify tags
        let mut tags = Vec::new();
        if self_ips.contains(&ip) {
            tags.push("Self".to_string());
        }
        if let Some(gw) = &iface.gateway {
            match ip {
                IpAddr::V4(ipv4) => {
                    if gw.ipv4.contains(&ipv4) {
                        tags.push("Gateway".to_string());
                    }
                }
                IpAddr::V6(ipv6) => {
                    if gw.ipv6.contains(&ipv6) {
                        tags.push("Gateway".to_string());
                    }
                }
            }
        }

        if iface.dns_servers.contains(&ip) {
            tags.push("DNS".to_string());
        }

        if !tags.is_empty() {
            ip_node.push(Tree::new(format!("Tags: {}", tags.join(", "))));
        }

        match ip {
            std::net::IpAddr::V4(_) => {
                v4.push(ip_node);
            },
            std::net::IpAddr::V6(_) => {
                v6.push(ip_node);
            },
        }
    }

    if !v4.leaves.is_empty() { root.push(v4); }
    if !v6.leaves.is_empty() { root.push(v6); }
    println!("{}", root);
}

fn print_neigh_table(table: &std::collections::HashMap<IpAddr, MacAddr>) {
    let iface = netdev::get_default_interface().unwrap();
    let self_ips: Vec<IpAddr> = iface.ip_addrs();

    let mut tbl = make_table(&["IP", "MAC", "Vendor", "Tags"]);

    let mut rows: Vec<_> = table.iter().collect();
    rows.sort_by(|(a, _), (b, _)| a.to_string().cmp(&b.to_string()));

    for (ip, mac) in rows {
        // Vendor lookup
        let vendor = if is_oui_db_initialized() && *mac != MacAddr::zero() && !mac.is_broadcast() {
            let oui_db = crate::db::oui::oui_db();
            oui_db
                .lookup_mac(mac)
                .map(|v| v.vendor_detail.as_deref().unwrap_or(&v.vendor).to_string())
        } else {
            None
        };

        // Classify tags
        let mut tags = Vec::new();

        // Self
        if self_ips.contains(ip) {
            tags.push("Self".to_string());
        }

        // Gateway
        if let Some(gw) = &iface.gateway {
            match ip {
                IpAddr::V4(ipv4) => {
                    if gw.ipv4.contains(ipv4) {
                        tags.push("Gateway".into());
                    }
                }
                IpAddr::V6(ipv6) => {
                    if gw.ipv6.contains(ipv6) {
                        tags.push("Gateway".into());
                    }
                }
            }
        }

        // DNS
        if iface.dns_servers.contains(ip) {
            tags.push("DNS".to_string());
        }

        tbl.add_row(vec![
            ip.to_string(),
            mac.to_string(),
            vendor.unwrap_or_else(|| "-".into()),
            if tags.is_empty() { "-".into() } else { tags.join(", ") },
        ]);
    }

    println!("{tbl}");
}
