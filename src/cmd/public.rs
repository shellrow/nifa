use anyhow::{Context, Result};
use netdev::Interface;
use mac_addr::MacAddr;
use reqwest::Client;
use termtree::Tree;
use std::time::Duration;

use crate::cli::{Cli, OutputFormat, PublicArgs};
use crate::db::oui::is_oui_db_initialized;
use crate::model::ipinfo::{CommonInfo, IpInfo, IpSide, PublicOut};
use crate::renderer::fmt_bps;
use crate::renderer::tree::tree_label;

const IPSTRUCT_URL: &str = "https://api.ipstruct.com/ip";
const IPSTRUCT_V4_URL: &str = "https://ipv4.ipstruct.com/ip";
//const IP_VERSION_4: &str = "v4";
const IP_VERSION_6: &str = "v6";

/// Show public IP information
pub async fn show_public_ip_info(_cli: &Cli, args: &PublicArgs) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(args.timeout.max(1)))
        .build()
        .context("build http client")?;

    let v4: Option<IpInfo>;
    let mut v6: Option<IpInfo> = None;

    if args.ipv4 {
        v4 = fetch_ip(&client, IPSTRUCT_V4_URL).await?;
    } else {
        let (any_res, v4_res) = tokio::join!(
            fetch_ip(&client, IPSTRUCT_URL),
            fetch_ip(&client, IPSTRUCT_V4_URL),
        );

        let any = any_res.unwrap_or(None);
        let v4opt = v4_res.unwrap_or(None);

        match any {
            Some(info) if is_ipv6(&info) => {
                v6 = Some(info);
                v4 = v4opt;
            }
            Some(info) => {
                v4 = Some(info);
            }
            None => {
                v4 = v4opt;
            }
        }
    }

    let out = build_public_out(v4, v6);

    let default_iface_opt = crate::net::iface::get_default_interface();

    match args.export {
        Some(export_date) => {
            crate::fs::export(
                export_date,
                args.output.as_deref(),
                &out,
            )?;
            return Ok(());
        }
        None => {
            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&out)?),
                _ => print_public_ip_tree(&out, default_iface_opt),
            }
        }
    }
    Ok(())
}

/// Fetch IP information from a given URL
async fn fetch_ip(client: &Client, url: &str) -> Result<Option<IpInfo>> {
    let resp = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("GET {}", url))?;
    if !resp.status().is_success() {
        anyhow::bail!("{} -> HTTP {}", url, resp.status());
    }
    let info: IpInfo = resp.json().await.context("parse json IpInfo")?;
    Ok(Some(info))
}

fn is_ipv6(info: &IpInfo) -> bool {
    info.ip_version == IP_VERSION_6 || info.ip_addr.contains(':')
}

fn build_public_out(v4: Option<IpInfo>, v6: Option<IpInfo>) -> PublicOut {
    // v4 or v6 is missing, cannot commonize
    if v4.is_none() || v6.is_none() {
        return PublicOut {
            common: None,
            ipv4: v4.as_ref().map(|i| IpSide {
                ip_addr: i.ip_addr.clone(),
                ip_addr_dec: i.ip_addr_dec.clone(),
                host_name: i.host_name.clone(),
                network: i.network.clone(),
                asn: Some(i.asn.clone()),
                as_name: Some(i.as_name.clone()),
                country_code: Some(i.country_code.clone()),
                country_name: Some(i.country_name.clone()),
            }),
            ipv6: v6.as_ref().map(|i| IpSide {
                ip_addr: i.ip_addr.clone(),
                ip_addr_dec: i.ip_addr_dec.clone(),
                host_name: i.host_name.clone(),
                network: i.network.clone(),
                asn: Some(i.asn.clone()),
                as_name: Some(i.as_name.clone()),
                country_code: Some(i.country_code.clone()),
                country_name: Some(i.country_name.clone()),
            }),
        };
    }

    let v4i = v4.as_ref().unwrap();
    let v6i = v6.as_ref().unwrap();

    let same_asn = v4i.asn == v6i.asn;
    let same_as_name = v4i.as_name == v6i.as_name;
    let same_cc = v4i.country_code == v6i.country_code;
    let same_country = v4i.country_name == v6i.country_name;

    // If all fields are the same, we can commonize
    if same_asn && same_as_name && same_cc && same_country {
        PublicOut {
            common: Some(CommonInfo {
                asn: v4i.asn.clone(),
                as_name: v4i.as_name.clone(),
                country_code: v4i.country_code.clone(),
                country_name: v4i.country_name.clone(),
            }),
            ipv4: Some(IpSide {
                ip_addr: v4i.ip_addr.clone(),
                ip_addr_dec: v4i.ip_addr_dec.clone(),
                host_name: v4i.host_name.clone(),
                network: v4i.network.clone(),
                asn: None,
                as_name: None,
                country_code: None,
                country_name: None,
            }),
            ipv6: Some(IpSide {
                ip_addr: v6i.ip_addr.clone(),
                ip_addr_dec: v6i.ip_addr_dec.clone(),
                host_name: v6i.host_name.clone(),
                network: v6i.network.clone(),
                asn: None,
                as_name: None,
                country_code: None,
                country_name: None,
            }),
        }
    } else {
        PublicOut {
            common: None,
            ipv4: Some(IpSide {
                ip_addr: v4i.ip_addr.clone(),
                ip_addr_dec: v4i.ip_addr_dec.clone(),
                host_name: v4i.host_name.clone(),
                network: v4i.network.clone(),
                asn: Some(v4i.asn.clone()),
                as_name: Some(v4i.as_name.clone()),
                country_code: Some(v4i.country_code.clone()),
                country_name: Some(v4i.country_name.clone()),
            }),
            ipv6: Some(IpSide {
                ip_addr: v6i.ip_addr.clone(),
                ip_addr_dec: v6i.ip_addr_dec.clone(),
                host_name: v6i.host_name.clone(),
                network: v6i.network.clone(),
                asn: Some(v6i.asn.clone()),
                as_name: Some(v6i.as_name.clone()),
                country_code: Some(v6i.country_code.clone()),
                country_name: Some(v6i.country_name.clone()),
            }),
        }
    }
}

fn print_public_ip_tree(out: &PublicOut, default_iface: Option<Interface>) {
    let host = crate::net::sys::hostname();
    let mut root = Tree::new(tree_label(format!("Public IPs on {}", host)));

    let mut v4node = Tree::new(tree_label("IPv4"));
    if let Some(i) = &out.ipv4 {
        v4node.push(Tree::new(tree_label(format!("IP: {}", i.ip_addr))));
        //v4node.push(Tree::new(tree_label(format!("Decimal: {}", i.ip_addr_dec))));
        //v4node.push(Tree::new(tree_label(format!("Host: {}", i.host_name))));
        v4node.push(Tree::new(tree_label(format!("Network: {}", i.network))));
        if out.common.is_none() {
            if let Some(asn) = &i.asn {
                v4node.push(Tree::new(tree_label(format!("ASN: {}", asn))));
            }
            if let Some(n) = &i.as_name {
                v4node.push(Tree::new(tree_label(format!("AS Name: {}", n))));
            }
            if let Some(cc) = &i.country_code {
                let cn = i.country_name.as_deref().unwrap_or("");
                v4node.push(Tree::new(tree_label(format!("Country: {} ({})", cn, cc))));
            }
        }
    } else {
        v4node.push(Tree::new(tree_label("(none)")));
    }
    root.push(v4node);

    let mut v6node = Tree::new(tree_label("IPv6"));
    if let Some(i) = &out.ipv6 {
        v6node.push(Tree::new(tree_label(format!("IP: {}", i.ip_addr))));
        //v6node.push(Tree::new(tree_label(format!("Decimal: {}", i.ip_addr_dec))));
        //v6node.push(Tree::new(tree_label(format!("Host: {}", i.host_name))));
        v6node.push(Tree::new(tree_label(format!("Network: {}", i.network))));
        if out.common.is_none() {
            if let Some(asn) = &i.asn {
                v6node.push(Tree::new(tree_label(format!("ASN: {}", asn))));
            }
            if let Some(n) = &i.as_name {
                v6node.push(Tree::new(tree_label(format!("AS Name: {}", n))));
            }
            if let Some(cc) = &i.country_code {
                let cn = i.country_name.as_deref().unwrap_or("");
                v6node.push(Tree::new(tree_label(format!("Country: {} ({})", cn, cc))));
            }
        }
    } else {
        v6node.push(Tree::new(tree_label("(none)")));
    }
    root.push(v6node);

    if let Some(c) = &out.common {
        let mut country_info = Tree::new(tree_label("Country"));
        country_info.push(Tree::new(tree_label(format!("Code: {}", c.country_code))));
        country_info.push(Tree::new(tree_label(format!("Name: {}", c.country_name))));
        root.push(country_info);

        let mut as_info = Tree::new(tree_label("AS Info"));
        as_info.push(Tree::new(tree_label(format!("ASN: {}", c.asn))));
        as_info.push(Tree::new(tree_label(format!("AS Name: {}", c.as_name))));
        root.push(as_info);
    }

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
        if_node.push(Tree::new(tree_label(format!(
            "State: {:?}",
            iface.oper_state
        ))));
        if let Some(mac) = &iface.mac_addr {
            if_node.push(Tree::new(tree_label(format!("MAC: {}", mac))));

            if is_oui_db_initialized() && *mac != MacAddr::zero() {
                let oui_db = crate::db::oui::oui_db();
                if let Some(vendor) = oui_db.lookup_mac(mac) {
                    let vendor_name = vendor.vendor_detail.as_deref().unwrap_or(&vendor.vendor);
                    if_node.push(Tree::new(format!("Vendor: {}", vendor_name)));
                }
            }
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

        let vpn_heuristic = crate::net::iface::detect_vpn_like(&iface);
        if vpn_heuristic.is_vpn_like {
            let mut heuristic_node = Tree::new(tree_label("Heuristic"));
            heuristic_node.push(Tree::new(format!(
                "VPN-like: {}",
                vpn_heuristic.is_vpn_like
            )));
            if_node.push(heuristic_node);
        }

        root.push(if_node);
    } else {
        root.push(Tree::new(tree_label("Default Interface: (not found)")));
    }

    println!("{}", root);
}
