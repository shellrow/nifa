use netdev::Interface;
use termtree::Tree;
use url::Url;
use mac_addr::MacAddr;
use anyhow::Result;

use crate::{cli::{Cli, SystemArgs}, db::oui::is_oui_db_initialized, net::sys::SysInfo, renderer::{fmt_bps, tree::tree_label}};

/// Show system network stack details
pub fn show_system_net_stack(_cli: &Cli, args: &SystemArgs) -> Result<()> {
    let sys_info = crate::net::sys::system_info();
    let default_iface_opt = crate::net::iface::get_default_interface();
    match args.export {
        Some(export_format) => {
            crate::fs::export(
                export_format,
                args.output.as_deref(),
                &sys_info,
            )?;
            return Ok(());
        }
        None => {
            match args.format {
                crate::cli::OutputFormat::Tree => {
                    print_system_with_default_iface(&sys_info, default_iface_opt)
                }
                crate::cli::OutputFormat::Json => {
                    crate::renderer::json::print_snapshot_json(&sys_info, default_iface_opt)
                }
                crate::cli::OutputFormat::Yaml => {
                    crate::renderer::yaml::print_snapshot_yaml(&sys_info, default_iface_opt)
                }
                _ => {
                    tracing::error!(
                        "Unsupported format for show system network stack: {:?}",
                        args.format
                    );
                }
            }
        }
    }
    Ok(())
}

/// Mask username/password in proxy URL for privacy
fn mask_proxy_url(raw: &str) -> String {
    if let Ok(mut url) = Url::parse(raw) {
        if url.password().is_some() || !url.username().is_empty() {
            let user = url.username().to_string();
            let _ = url.set_username(&user).ok();
            let _ = url.set_password(Some("*****")).ok();
        }
        return url.to_string();
    }
    raw.to_string()
}

fn print_system_with_default_iface(sys: &SysInfo, default_iface: Option<Interface>) {
    let mut root = Tree::new(tree_label(format!(
        "System Information on {}",
        sys.hostname
    )));

    // ---- System ----
    let mut sys_node = Tree::new(tree_label("System"));
    sys_node.push(Tree::new(tree_label(format!("OS Type: {}", sys.os_type))));
    sys_node.push(Tree::new(tree_label(format!(
        "Version: {}",
        sys.os_version
    ))));
    if let Some(kv) = &sys.kernel_version {
        sys_node.push(Tree::new(tree_label(format!("Kernel: {}", kv))));
    }
    sys_node.push(Tree::new(tree_label(format!("Edition: {}", sys.edition))));
    sys_node.push(Tree::new(tree_label(format!("Codename: {}", sys.codename))));
    sys_node.push(Tree::new(tree_label(format!("Bitness: {}", sys.bitness))));
    sys_node.push(Tree::new(tree_label(format!(
        "Architecture: {}",
        sys.architecture
    ))));

    // ---- Proxy (env) ----
    let px = crate::net::sys::collect_proxy_env();
    let mut px_node = Tree::new(tree_label("Proxy (env)"));
    px_node.push(Tree::new(format!(
        "HTTP_PROXY: {}",
        px.http
            .as_deref()
            .map(mask_proxy_url)
            .unwrap_or_else(|| "(none)".into())
    )));
    px_node.push(Tree::new(format!(
        "HTTPS_PROXY: {}",
        px.https
            .as_deref()
            .map(mask_proxy_url)
            .unwrap_or_else(|| "(none)".into())
    )));
    px_node.push(Tree::new(format!(
        "ALL_PROXY: {}",
        px.all
            .as_deref()
            .map(mask_proxy_url)
            .unwrap_or_else(|| "(none)".into())
    )));
    if let Some(np) = px.no_proxy.as_deref() {
        let mut np_node = Tree::new(tree_label("NO_PROXY"));
        // Split by comma and trim spaces, ignore empty parts
        for (i, part) in np
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .enumerate()
        {
            // Limit to first 20 entries
            if i < 20 {
                np_node.push(Tree::new(part.to_string()));
            } else {
                np_node.push(Tree::new(format!(
                    "(+{} more)",
                    np.split(',').count().saturating_sub(20)
                )));
                break;
            }
        }
        px_node.push(np_node);
    } else {
        px_node.push(Tree::new(tree_label("NO_PROXY: (none)")));
    }
    sys_node.push(px_node);

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
