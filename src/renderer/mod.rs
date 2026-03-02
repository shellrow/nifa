use anyhow::Result;
use comfy_table::{
    Cell, ContentArrangement, Table, modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL,
};
use serde::Serialize;
use termtree::Tree;

use crate::cli::{OutputArgs, OutputFormat};
use crate::model::{
    IfDetail, IfSummary, IpAddrEntry, NeighEntry, RouteEntry, SockEntry, SystemSummary,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedFormat {
    Tree,
    Table,
    Json,
    Yaml,
}

#[derive(Debug, Serialize)]
struct AddrByIface {
    iface: String,
    if_type: String,
    ipv4: Vec<String>,
    ipv6: Vec<String>,
}

pub fn resolve_output_format(out: &OutputArgs) -> ResolvedFormat {
    match out.format {
        OutputFormat::Tree => ResolvedFormat::Tree,
        OutputFormat::Table => ResolvedFormat::Table,
        OutputFormat::Json => ResolvedFormat::Json,
        OutputFormat::Yaml => ResolvedFormat::Yaml,
        OutputFormat::Auto => {
            let width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(120);
            if out.wide || width >= 110 {
                ResolvedFormat::Table
            } else {
                ResolvedFormat::Tree
            }
        }
    }
}

pub fn fmt_bps(bps: u64) -> String {
    const K: f64 = 1_000.0;
    let b = bps as f64;
    if b >= K * K * K {
        format!("{:.2} Gb/s", b / (K * K * K))
    } else if b >= K * K {
        format!("{:.2} Mb/s", b / (K * K))
    } else if b >= K {
        format!("{:.2} Kb/s", b / K)
    } else {
        format!("{} b/s", bps)
    }
}

pub fn fmt_flags(flags: u32) -> String {
    format!("0x{flags:08X}")
}

fn make_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(headers.iter().map(|h| Cell::new(*h)).collect::<Vec<_>>());
    table
}

fn print_json<T: Serialize + ?Sized>(data: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(data)?);
    Ok(())
}

fn print_yaml<T: Serialize + ?Sized>(data: &T) -> Result<()> {
    println!("{}", serde_yaml::to_string(data)?);
    Ok(())
}

pub fn render_if_summaries(data: &[IfSummary], out: &OutputArgs) -> Result<()> {
    match resolve_output_format(out) {
        ResolvedFormat::Json => print_json(data),
        ResolvedFormat::Yaml => print_yaml(data),
        ResolvedFormat::Tree => {
            let mut root = Tree::new("Interfaces".to_string());
            for item in data {
                let mut node = Tree::new(format!(
                    "{}{}",
                    item.name,
                    if item.is_default { " (default)" } else { "" }
                ));
                node.push(Tree::new(format!("Index: {}", item.index)));
                node.push(Tree::new(format!("Type: {}", item.if_type)));
                node.push(Tree::new(format!("State: {}", item.state)));
                node.push(Tree::new(format!(
                    "MAC: {}",
                    item.mac.as_deref().unwrap_or("-")
                )));
                node.push(Tree::new(format!(
                    "MTU: {}",
                    item.mtu
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "-".into())
                )));
                let mut ipv4 = Tree::new("IPv4".to_string());
                if item.ipv4_addrs.is_empty() {
                    ipv4.push(Tree::new("(none)".to_string()));
                } else {
                    for addr in &item.ipv4_addrs {
                        ipv4.push(Tree::new(addr.clone()));
                    }
                }
                node.push(ipv4);

                let mut ipv6 = Tree::new("IPv6".to_string());
                if item.ipv6_addrs.is_empty() {
                    ipv6.push(Tree::new("(none)".to_string()));
                } else {
                    for addr in &item.ipv6_addrs {
                        ipv6.push(Tree::new(addr.clone()));
                    }
                }
                node.push(ipv6);

                if item.gateway_mac.is_some()
                    || !item.gateway_ipv4.is_empty()
                    || !item.gateway_ipv6.is_empty()
                {
                    let mut gateway = Tree::new("Gateway".to_string());
                    if let Some(mac) = &item.gateway_mac {
                        gateway.push(Tree::new(format!("MAC: {}", mac)));
                    }

                    let mut gw4 = Tree::new("IPv4".to_string());
                    if item.gateway_ipv4.is_empty() {
                        gw4.push(Tree::new("(none)".to_string()));
                    } else {
                        for ip in &item.gateway_ipv4 {
                            gw4.push(Tree::new(ip.clone()));
                        }
                    }
                    gateway.push(gw4);

                    let mut gw6 = Tree::new("IPv6".to_string());
                    if item.gateway_ipv6.is_empty() {
                        gw6.push(Tree::new("(none)".to_string()));
                    } else {
                        for ip in &item.gateway_ipv6 {
                            gw6.push(Tree::new(ip.clone()));
                        }
                    }
                    gateway.push(gw6);

                    node.push(gateway);
                }
                if let Some(vendor) = &item.vendor {
                    node.push(Tree::new(format!("Vendor: {vendor}")));
                }
                root.push(node);
            }
            println!("{root}");
            Ok(())
        }
        ResolvedFormat::Table => {
            let mut table = make_table(&[
                "INDEX", "NAME", "TYPE", "STATE", "MAC", "MTU", "IPv4", "IPv6", "VENDOR",
            ]);
            for item in data {
                table.add_row(vec![
                    item.index.to_string(),
                    item.name.clone(),
                    item.if_type.clone(),
                    item.state.clone(),
                    item.mac.clone().unwrap_or_else(|| "-".into()),
                    item.mtu
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "-".into()),
                    item.ipv4_addrs.len().to_string(),
                    item.ipv6_addrs.len().to_string(),
                    item.vendor.clone().unwrap_or_else(|| "-".into()),
                ]);
            }
            println!("{table}");
            Ok(())
        }
    }
}

pub fn render_if_detail(data: &IfDetail, out: &OutputArgs) -> Result<()> {
    match resolve_output_format(out) {
        ResolvedFormat::Json => print_json(data),
        ResolvedFormat::Yaml => print_yaml(data),
        ResolvedFormat::Table => {
            let mut table = make_table(&["FIELD", "VALUE"]);
            table.add_row(vec!["name".into(), data.summary.name.clone()]);
            table.add_row(vec!["index".into(), data.summary.index.to_string()]);
            table.add_row(vec![
                "friendly_name".into(),
                data.friendly_name.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec!["type".into(), data.summary.if_type.clone()]);
            table.add_row(vec!["state".into(), data.summary.state.clone()]);
            table.add_row(vec![
                "mac".into(),
                data.summary.mac.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "mtu".into(),
                data.summary
                    .mtu
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec!["flags".into(), data.flags.clone()]);
            table.add_row(vec![
                "tx_speed".into(),
                data.tx_speed.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "rx_speed".into(),
                data.rx_speed.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "gateway_mac".into(),
                data.gateway_mac.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "gateway_ipv4".into(),
                if data.gateway_ipv4.is_empty() {
                    "-".into()
                } else {
                    data.gateway_ipv4.join(", ")
                },
            ]);
            table.add_row(vec![
                "gateway_ipv6".into(),
                if data.gateway_ipv6.is_empty() {
                    "-".into()
                } else {
                    data.gateway_ipv6.join(", ")
                },
            ]);
            table.add_row(vec![
                "dns_servers".into(),
                if data.dns_servers.is_empty() {
                    "-".into()
                } else {
                    data.dns_servers.join(", ")
                },
            ]);
            table.add_row(vec![
                "stats_rx_bytes".into(),
                data.stats_rx_bytes
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "stats_tx_bytes".into(),
                data.stats_tx_bytes
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec!["vpn_like".into(), data.vpn_like.to_string()]);
            println!("{table}");
            Ok(())
        }
        ResolvedFormat::Tree => {
            let mut root = Tree::new(data.summary.name.clone());
            root.push(Tree::new(format!("Index: {}", data.summary.index)));
            if let Some(v) = &data.friendly_name {
                root.push(Tree::new(format!("Friendly Name: {v}")));
            }
            root.push(Tree::new(format!("Type: {}", data.summary.if_type)));
            root.push(Tree::new(format!("State: {}", data.summary.state)));
            root.push(Tree::new(format!(
                "MAC: {}",
                data.summary.mac.as_deref().unwrap_or("-")
            )));
            root.push(Tree::new(format!(
                "MTU: {}",
                data.summary
                    .mtu
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "-".into())
            )));
            root.push(Tree::new(format!("Flags: {}", data.flags)));

            if let Some(v) = &data.summary.vendor {
                root.push(Tree::new(format!("Vendor: {v}")));
            }
            if let Some(v) = &data.description {
                root.push(Tree::new(format!("Description: {v}")));
            }
            if let Some(v) = &data.tx_speed {
                root.push(Tree::new(format!("TX Speed: {v}")));
            }
            if let Some(v) = &data.rx_speed {
                root.push(Tree::new(format!("RX Speed: {v}")));
            }

            let mut v4 = Tree::new("IPv4".to_string());
            for a in &data.ipv4 {
                v4.push(Tree::new(a.clone()));
            }
            if !data.ipv4.is_empty() {
                root.push(v4);
            }

            let mut v6 = Tree::new("IPv6".to_string());
            for a in &data.ipv6_scoped {
                v6.push(Tree::new(a.clone()));
            }
            if !data.ipv6_scoped.is_empty() {
                root.push(v6);
            }

            let mut dns = Tree::new("DNS".to_string());
            if data.dns_servers.is_empty() {
                dns.push(Tree::new("(none)".to_string()));
            } else {
                for d in &data.dns_servers {
                    dns.push(Tree::new(d.clone()));
                }
            }
            root.push(dns);

            if data.gateway_mac.is_some()
                || !data.gateway_ipv4.is_empty()
                || !data.gateway_ipv6.is_empty()
            {
                let mut gateway = Tree::new("Gateway".to_string());
                if let Some(mac) = &data.gateway_mac {
                    gateway.push(Tree::new(format!("MAC: {}", mac)));
                }

                if !data.gateway_ipv4.is_empty() {
                    let mut gw4 = Tree::new("IPv4".to_string());
                    for ip in &data.gateway_ipv4 {
                        gw4.push(Tree::new(ip.clone()));
                    }
                    gateway.push(gw4);
                }
                if !data.gateway_ipv6.is_empty() {
                    let mut gw6 = Tree::new("IPv6".to_string());
                    for ip in &data.gateway_ipv6 {
                        gw6.push(Tree::new(ip.clone()));
                    }
                    gateway.push(gw6);
                }
                root.push(gateway);
            }

            if data.stats_rx_bytes.is_some() || data.stats_tx_bytes.is_some() {
                let mut stats = Tree::new("Statistics (snapshot)".to_string());
                if let Some(rx) = data.stats_rx_bytes {
                    stats.push(Tree::new(format!("RX bytes: {}", rx)));
                }
                if let Some(tx) = data.stats_tx_bytes {
                    stats.push(Tree::new(format!("TX bytes: {}", tx)));
                }
                root.push(stats);
            }

            root.push(Tree::new(format!("VPN-like: {}", data.vpn_like)));

            println!("{root}");
            Ok(())
        }
    }
}

pub fn render_ip_entries(data: &[IpAddrEntry], out: &OutputArgs) -> Result<()> {
    match resolve_output_format(out) {
        ResolvedFormat::Json => {
            let grouped = group_ip_entries_by_iface(data);
            print_json(&grouped)
        }
        ResolvedFormat::Yaml => {
            let grouped = group_ip_entries_by_iface(data);
            print_yaml(&grouped)
        }
        ResolvedFormat::Tree => {
            let mut root = Tree::new("IP Addresses".to_string());
            let mut ifaces: std::collections::BTreeMap<String, (String, Vec<String>, Vec<String>)> =
                std::collections::BTreeMap::new();

            for e in data {
                let entry = ifaces
                    .entry(e.iface.clone())
                    .or_insert_with(|| (e.if_type.clone(), Vec::new(), Vec::new()));
                let addr_label = if let Some(scope) = e.scope_id {
                    format!("{}/{} (scope_id={scope})", e.address, e.prefix_len)
                } else {
                    format!("{}/{}", e.address, e.prefix_len)
                };
                if e.family == "ipv4" {
                    entry.1.push(addr_label);
                } else {
                    entry.2.push(addr_label);
                }
            }

            for (iface, (if_type, v4_addrs, v6_addrs)) in ifaces {
                let mut iface_node = Tree::new(iface);
                iface_node.push(Tree::new(format!("Type: {}", if_type)));

                let mut v4 = Tree::new("IPv4".to_string());
                if v4_addrs.is_empty() {
                    v4.push(Tree::new("(none)".to_string()));
                } else {
                    for addr in v4_addrs {
                        v4.push(Tree::new(addr));
                    }
                }
                iface_node.push(v4);

                let mut v6 = Tree::new("IPv6".to_string());
                if v6_addrs.is_empty() {
                    v6.push(Tree::new("(none)".to_string()));
                } else {
                    for addr in v6_addrs {
                        v6.push(Tree::new(addr));
                    }
                }
                iface_node.push(v6);

                root.push(iface_node);
            }
            println!("{root}");
            Ok(())
        }
        ResolvedFormat::Table => {
            let mut table = make_table(&["IFACE", "TYPE", "FAMILY", "ADDRESS", "PREFIX", "SCOPE"]);
            for e in data {
                table.add_row(vec![
                    e.iface.clone(),
                    e.if_type.clone(),
                    e.family.clone(),
                    e.address.clone(),
                    e.prefix_len.to_string(),
                    e.scope_id
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "-".into()),
                ]);
            }
            println!("{table}");
            Ok(())
        }
    }
}

fn group_ip_entries_by_iface(data: &[IpAddrEntry]) -> Vec<AddrByIface> {
    let mut grouped: std::collections::BTreeMap<String, (String, Vec<String>, Vec<String>)> =
        std::collections::BTreeMap::new();

    for e in data {
        let entry = grouped
            .entry(e.iface.clone())
            .or_insert_with(|| (e.if_type.clone(), Vec::new(), Vec::new()));
        let addr = format!("{}/{}", e.address, e.prefix_len);
        if e.family == "ipv4" {
            entry.1.push(addr);
        } else {
            entry.2.push(addr);
        }
    }

    grouped
        .into_iter()
        .map(|(iface, (if_type, mut ipv4, mut ipv6))| {
            ipv4.sort();
            ipv6.sort();
            AddrByIface {
                iface,
                if_type,
                ipv4,
                ipv6,
            }
        })
        .collect()
}

pub fn render_link_entries(data: &[IfDetail], out: &OutputArgs) -> Result<()> {
    match resolve_output_format(out) {
        ResolvedFormat::Json => print_json(data),
        ResolvedFormat::Yaml => print_yaml(data),
        ResolvedFormat::Tree => {
            let mut root = Tree::new("Link Layer".to_string());
            for item in data {
                let mut node = Tree::new(item.summary.name.clone());
                node.push(Tree::new(format!("Index: {}", item.summary.index)));
                node.push(Tree::new(format!("Type: {}", item.summary.if_type)));
                node.push(Tree::new(format!(
                    "MAC: {}",
                    item.summary.mac.as_deref().unwrap_or("-")
                )));
                node.push(Tree::new(format!(
                    "MTU: {}",
                    item.summary
                        .mtu
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "-".into())
                )));
                node.push(Tree::new(format!("State: {}", item.summary.state)));
                node.push(Tree::new(format!("Flags: {}", item.flags)));
                node.push(Tree::new(format!(
                    "Speed(TX/RX): {}/{}",
                    item.tx_speed.as_deref().unwrap_or("-"),
                    item.rx_speed.as_deref().unwrap_or("-")
                )));
                root.push(node);
            }
            println!("{root}");
            Ok(())
        }
        ResolvedFormat::Table => {
            let mut table = make_table(&[
                "INDEX", "IFACE", "TYPE", "MAC", "MTU", "STATE", "FLAGS", "SPEED",
            ]);
            for item in data {
                table.add_row(vec![
                    item.summary.index.to_string(),
                    item.summary.name.clone(),
                    item.summary.if_type.clone(),
                    item.summary.mac.clone().unwrap_or_else(|| "-".into()),
                    item.summary
                        .mtu
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "-".into()),
                    item.summary.state.clone(),
                    item.flags.clone(),
                    format!(
                        "{}/{}",
                        item.tx_speed.as_deref().unwrap_or("-"),
                        item.rx_speed.as_deref().unwrap_or("-")
                    ),
                ]);
            }
            println!("{table}");
            Ok(())
        }
    }
}

pub fn render_routes(data: &[RouteEntry], out: &OutputArgs) -> Result<()> {
    match resolve_output_format(out) {
        ResolvedFormat::Json => print_json(data),
        ResolvedFormat::Yaml => print_yaml(data),
        ResolvedFormat::Tree => {
            let mut root = Tree::new("Routes".to_string());
            for r in data {
                let mut node = Tree::new(format!("{} {}", r.family, r.destination));
                node.push(Tree::new(format!(
                    "Gateway: {}",
                    r.gateway.as_deref().unwrap_or("-")
                )));
                node.push(Tree::new(format!(
                    "Interface: {}",
                    r.interface.as_deref().unwrap_or("-")
                )));
                node.push(Tree::new(format!(
                    "Metric: {}",
                    r.metric
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "-".into())
                )));
                if !r.flags.is_empty() {
                    node.push(Tree::new(format!("Flags: {}", r.flags.join(","))));
                }
                if let Some(d) = &r.detail {
                    node.push(Tree::new(format!("Detail: {d}")));
                }
                root.push(node);
            }
            println!("{root}");
            Ok(())
        }
        ResolvedFormat::Table => {
            let mut table =
                make_table(&["FAMILY", "DESTINATION", "GATEWAY", "DEV", "METRIC", "FLAGS"]);
            for r in data {
                table.add_row(vec![
                    r.family.clone(),
                    r.destination.clone(),
                    r.gateway.clone().unwrap_or_else(|| "-".into()),
                    r.interface.clone().unwrap_or_else(|| "-".into()),
                    r.metric
                        .map(|m| m.to_string())
                        .unwrap_or_else(|| "-".into()),
                    if r.flags.is_empty() {
                        "-".into()
                    } else {
                        r.flags.join("")
                    },
                ]);
            }
            println!("{table}");
            Ok(())
        }
    }
}

pub fn render_neighbors(data: &[NeighEntry], out: &OutputArgs) -> Result<()> {
    match resolve_output_format(out) {
        ResolvedFormat::Json => print_json(data),
        ResolvedFormat::Yaml => print_yaml(data),
        ResolvedFormat::Tree => {
            let mut root = Tree::new("Neighbors".to_string());
            for n in data {
                let mut node = Tree::new(format!("{} {}", n.family, n.ip));
                node.push(Tree::new(format!("MAC: {}", n.mac)));
                if let Some(v) = &n.vendor {
                    node.push(Tree::new(format!("Vendor: {v}")));
                }
                if !n.tags.is_empty() {
                    node.push(Tree::new(format!("Tags: {}", n.tags.join(", "))));
                }
                root.push(node);
            }
            println!("{root}");
            Ok(())
        }
        ResolvedFormat::Table => {
            let mut table = make_table(&["FAMILY", "IP", "MAC", "VENDOR", "TAGS"]);
            for n in data {
                table.add_row(vec![
                    n.family.clone(),
                    n.ip.clone(),
                    n.mac.clone(),
                    n.vendor.clone().unwrap_or_else(|| "-".into()),
                    if n.tags.is_empty() {
                        "-".into()
                    } else {
                        n.tags.join(", ")
                    },
                ]);
            }
            println!("{table}");
            Ok(())
        }
    }
}

pub fn render_sockets(data: &[SockEntry], out: &OutputArgs, include_pid: bool) -> Result<()> {
    match resolve_output_format(out) {
        ResolvedFormat::Json => print_json(data),
        ResolvedFormat::Yaml => print_yaml(data),
        ResolvedFormat::Tree => {
            let mut root = Tree::new("Sockets".to_string());
            for s in data {
                let mut node = Tree::new(format!(
                    "{} [{}] {} -> {}",
                    s.proto,
                    s.family,
                    s.local,
                    s.remote.as_deref().unwrap_or("-")
                ));
                if let Some(state) = &s.state {
                    node.push(Tree::new(format!("State: {state}")));
                }
                if include_pid {
                    node.push(Tree::new(format!(
                        "PID: {}",
                        s.pid.map(|v| v.to_string()).unwrap_or_else(|| "-".into())
                    )));
                    node.push(Tree::new(format!(
                        "Process: {}",
                        s.process.as_deref().unwrap_or("-")
                    )));
                }
                root.push(node);
            }
            println!("{root}");
            Ok(())
        }
        ResolvedFormat::Table => {
            let mut headers = vec!["PROTO", "FAMILY", "LOCAL", "REMOTE", "STATE"];
            if include_pid {
                headers.push("PID");
                headers.push("PROCESS");
            }
            let mut table = make_table(&headers);
            for s in data {
                let mut row = vec![
                    s.proto.clone(),
                    s.family.clone(),
                    s.local.clone(),
                    s.remote.clone().unwrap_or_else(|| "-".into()),
                    s.state.clone().unwrap_or_else(|| "-".into()),
                ];
                if include_pid {
                    row.push(s.pid.map(|v| v.to_string()).unwrap_or_else(|| "-".into()));
                    row.push(s.process.clone().unwrap_or_else(|| "-".into()));
                }
                table.add_row(row);
            }
            println!("{table}");
            Ok(())
        }
    }
}

pub fn render_system_summary(data: &SystemSummary, out: &OutputArgs) -> Result<()> {
    match resolve_output_format(out) {
        ResolvedFormat::Json => print_json(data),
        ResolvedFormat::Yaml => print_yaml(data),
        ResolvedFormat::Table => {
            let mut table = make_table(&["FIELD", "VALUE"]);
            table.add_row(vec!["hostname".into(), data.hostname.clone()]);
            table.add_row(vec!["os".into(), data.os.clone()]);
            table.add_row(vec![
                "kernel".into(),
                data.kernel_version.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "default_interface".into(),
                data.default_interface.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "default_gateway".into(),
                data.default_gateway.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "dns_servers".into(),
                if data.dns_servers.is_empty() {
                    "-".into()
                } else {
                    data.dns_servers.join(", ")
                },
            ]);
            table.add_row(vec![
                "http_proxy".into(),
                data.proxy_http.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "https_proxy".into(),
                data.proxy_https.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "all_proxy".into(),
                data.proxy_all.clone().unwrap_or_else(|| "-".into()),
            ]);
            table.add_row(vec![
                "no_proxy".into(),
                data.proxy_no_proxy.clone().unwrap_or_else(|| "-".into()),
            ]);
            println!("{table}");
            Ok(())
        }
        ResolvedFormat::Tree => {
            let mut root = Tree::new(format!("System {}", data.hostname));
            root.push(Tree::new(format!("OS: {}", data.os)));
            root.push(Tree::new(format!(
                "Kernel: {}",
                data.kernel_version.as_deref().unwrap_or("-")
            )));
            root.push(Tree::new(format!(
                "Default Interface: {}",
                data.default_interface.as_deref().unwrap_or("-")
            )));
            root.push(Tree::new(format!(
                "Default Gateway: {}",
                data.default_gateway.as_deref().unwrap_or("-")
            )));

            let mut dns = Tree::new("DNS".to_string());
            if data.dns_servers.is_empty() {
                dns.push(Tree::new("-".to_string()));
            } else {
                for item in &data.dns_servers {
                    dns.push(Tree::new(item.clone()));
                }
            }
            root.push(dns);

            let mut proxy = Tree::new("Proxy".to_string());
            proxy.push(Tree::new(format!(
                "HTTP: {}",
                data.proxy_http.as_deref().unwrap_or("-")
            )));
            proxy.push(Tree::new(format!(
                "HTTPS: {}",
                data.proxy_https.as_deref().unwrap_or("-")
            )));
            proxy.push(Tree::new(format!(
                "ALL: {}",
                data.proxy_all.as_deref().unwrap_or("-")
            )));
            proxy.push(Tree::new(format!(
                "NO_PROXY: {}",
                data.proxy_no_proxy.as_deref().unwrap_or("-")
            )));
            root.push(proxy);

            println!("{root}");
            Ok(())
        }
    }
}
