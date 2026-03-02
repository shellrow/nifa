use std::net::IpAddr;

use anyhow::Result;
use mac_addr::MacAddr;

use crate::cli::NeighArgs;
use crate::model::NeighEntry;

pub fn run(args: &NeighArgs) -> Result<()> {
    if args.vendor {
        crate::db::oui::init_oui_db()?;
    }

    let table = crate::net::neigh::get_neighbor_table()?;
    let default_iface = crate::net::iface::get_primary_interface();
    let self_ips: Vec<IpAddr> = default_iface
        .as_ref()
        .map(|i| i.ip_addrs())
        .unwrap_or_default();

    let mut entries: Vec<NeighEntry> = table
        .into_iter()
        .map(|(ip, mac)| {
            let vendor = if args.vendor && mac != MacAddr::zero() && !mac.is_broadcast() {
                let oui_db = crate::db::oui::oui_db();
                oui_db
                    .lookup_mac(&mac)
                    .map(|v| v.vendor_detail.as_deref().unwrap_or(&v.vendor).to_string())
            } else {
                None
            };

            let mut tags = Vec::new();
            if self_ips.contains(&ip) {
                tags.push("Self".to_string());
            }
            if let Some(iface) = &default_iface {
                if let Some(gw) = &iface.gateway {
                    match ip {
                        IpAddr::V4(v4) if gw.ipv4.contains(&v4) => tags.push("Gateway".to_string()),
                        IpAddr::V6(v6) if gw.ipv6.contains(&v6) => tags.push("Gateway".to_string()),
                        _ => {}
                    }
                }
                if iface.dns_servers.contains(&ip) {
                    tags.push("DNS".to_string());
                }
            }

            NeighEntry {
                iface: default_iface.as_ref().map(|i| i.name.clone()),
                family: if ip.is_ipv4() {
                    "ipv4".to_string()
                } else {
                    "ipv6".to_string()
                },
                ip: ip.to_string(),
                mac: mac.to_string(),
                vendor,
                tags,
            }
        })
        .collect();

    if args.ipv4 {
        entries.retain(|e| e.family == "ipv4");
    }
    if args.ipv6 {
        entries.retain(|e| e.family == "ipv6");
    }
    if let Some(iface_name) = &args.iface {
        entries.retain(|e| e.iface.as_deref() == Some(iface_name.as_str()));
    }
    entries.sort_by(|a, b| {
        a.family
            .cmp(&b.family)
            .then(a.ip.cmp(&b.ip))
            .then(a.mac.cmp(&b.mac))
    });

    crate::renderer::render_neighbors(&entries, &args.out)
}
