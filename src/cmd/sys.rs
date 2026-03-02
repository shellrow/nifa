use anyhow::Result;

use crate::cli::SysArgs;
use crate::model::SystemSummary;

pub fn run(args: &SysArgs) -> Result<()> {
    let sys_info = crate::net::sys::system_info();
    let default_iface = crate::net::iface::get_primary_interface();

    let dns_servers = if args.summary || args.proxy {
        Vec::new()
    } else {
        default_iface
            .as_ref()
            .map(|iface| iface.dns_servers.iter().map(|d| d.to_string()).collect())
            .unwrap_or_default()
    };

    let (proxy_http, proxy_https, proxy_all, proxy_no_proxy) = if args.summary || args.dns {
        (None, None, None, None)
    } else {
        (
            sys_info.proxy.http.clone(),
            sys_info.proxy.https.clone(),
            sys_info.proxy.all.clone(),
            sys_info.proxy.no_proxy.clone(),
        )
    };

    let summary = SystemSummary {
        hostname: sys_info.hostname,
        os: format!("{} {}", sys_info.os_type, sys_info.os_version),
        kernel_version: sys_info.kernel_version,
        default_interface: default_iface.as_ref().map(|iface| iface.name.clone()),
        default_gateway: default_iface.as_ref().and_then(|iface| {
            iface
                .gateway
                .as_ref()
                .and_then(|gw| gw.ipv4.first().map(|ip| ip.to_string()))
                .or_else(|| {
                    iface
                        .gateway
                        .as_ref()
                        .and_then(|gw| gw.ipv6.first().map(|ip| ip.to_string()))
                })
        }),
        dns_servers,
        proxy_http,
        proxy_https,
        proxy_all,
        proxy_no_proxy,
    };

    crate::renderer::render_system_summary(&summary, &args.out)
}
