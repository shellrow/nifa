use anyhow::Result;
use netroute::{RouteFamily, RouteFlag};

use crate::cli::RouteArgs;
use crate::model::RouteEntry;

pub fn run(args: &RouteArgs) -> Result<()> {
    let mut routes = crate::net::route::list_routes()?;

    if args.ipv4 {
        routes.retain(|r| r.family == RouteFamily::Ipv4);
    }
    if args.ipv6 {
        routes.retain(|r| r.family == RouteFamily::Ipv6);
    }
    if args.default {
        routes.retain(|r| r.destination.addr.is_unspecified());
    }

    let mut entries: Vec<RouteEntry> = routes
        .into_iter()
        .map(|r| RouteEntry {
            family: match r.family {
                RouteFamily::Ipv4 => "ipv4".to_string(),
                RouteFamily::Ipv6 => "ipv6".to_string(),
            },
            destination: r.destination.to_string(),
            gateway: r.gateway.map(|g| g.to_string()),
            interface: r.ifname,
            metric: r.metric,
            flags: r.flags.iter().map(RouteFlag::short).collect(),
            detail: if args.detail {
                Some(format!(
                    "proto={:?},scope={:?},table={:?},on_link={},ifindex={:?},lifetime_ms={:?}",
                    r.protocol, r.scope, r.table, r.on_link, r.ifindex, r.lifetime_ms
                ))
            } else {
                None
            },
        })
        .collect();

    entries.sort_by(|a, b| {
        a.family
            .cmp(&b.family)
            .then(a.destination.cmp(&b.destination))
            .then(a.interface.cmp(&b.interface))
    });

    crate::renderer::render_routes(&entries, &args.out)
}
