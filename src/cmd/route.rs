use anyhow::Result;
use crate::cli::{Cli, OutputFormat, RouteArgs, RouteFamilyOpt};
use crate::net::route;
use crate::renderer::table::make_table;
use netroute::{RouteEntry, RouteFamily, RouteFlag};
use termtree::Tree;
use crate::renderer::tree::tree_label;

pub fn show_route(_cli: &Cli, args: &RouteArgs) -> Result<()> {
    let mut routes = route::list_routes()?;

    // family filter
    routes.retain(|r| match args.family {
        RouteFamilyOpt::All => true,
        RouteFamilyOpt::Ipv4 => r.family == RouteFamily::Ipv4,
        RouteFamilyOpt::Ipv6 => r.family == RouteFamily::Ipv6,
    });

    match args.export {
        Some(export_format) => {
            crate::fs::export(
                export_format,
                args.output.as_deref(),
                &routes,
            )?;
            return Ok(());
        }
        None => {
            match args.format {
                OutputFormat::Tree => print_route_tree(&routes),
                OutputFormat::Table => print_route_table(&routes),
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&routes)?),
                OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&routes)?),
            }
        }
    }
    Ok(())
}

fn print_route_tree(routes: &[RouteEntry]) {
    let host = crate::net::sys::hostname();
    let mut root = Tree::new(tree_label(format!("Routing Table on {}", host)));

    let mut v4 = Tree::new(tree_label("IPv4"));
    let mut v6 = Tree::new(tree_label("IPv6"));

    for r in routes {
        let mut node = Tree::new(tree_label(format!("{}", r.destination)));
        if let Some(gw) = r.gateway {
            node.push(Tree::new(format!("via {}", gw)));
        } else if r.on_link {
            node.push(Tree::new(tree_label("via link")));
        }

        if let Some(name) = r.ifname.as_ref() {
            node.push(Tree::new(format!("dev {}", name)));
        } else if let Some(idx) = r.ifindex {
            node.push(Tree::new(format!("ifindex {}", idx)));
        }

        if let Some(m) = r.metric {
            node.push(Tree::new(format!("metric {}", m)));
        }

        if !r.flags.is_empty() {
            let short = flags_short(r);
            node.push(Tree::new(format!("flags {}", short)));
        }

        if let Some(p) = r.protocol.as_ref() {
            node.push(Tree::new(format!("proto {:?}", p)));
        }
        if let Some(s) = r.scope.as_ref() {
            node.push(Tree::new(format!("scope {:?}", s)));
        }
        if let Some(tbl) = r.table {
            node.push(Tree::new(format!("table {}", tbl)));
        }
        if let Some(ms) = r.lifetime_ms {
            node.push(Tree::new(format!("lifetime {}ms", ms)));
        }

        match r.family {
            RouteFamily::Ipv4 => {
                v4.push(node);
            },
            RouteFamily::Ipv6 => {
                v6.push(node);
            },
        }
    }

    if !v4.leaves.is_empty() { root.push(v4); }
    if !v6.leaves.is_empty() { root.push(v6); }
    println!("{}", root);
}

fn print_route_table(routes: &[RouteEntry]) {
    let mut table = make_table(&["FAMILY", "DESTINATION", "VIA/NH", "DEV", "METRIC", "FLAGS"]);

    for r in routes {
        let fam = match r.family { RouteFamily::Ipv4 => "v4", RouteFamily::Ipv6 => "v6" };
        let via = r.gateway.map(|g| g.to_string())
            .unwrap_or_else(|| if r.on_link { "link".into() } else { "-".into() });
        let dev = r.ifname.as_deref().unwrap_or("-");
        let metric = r.metric.map(|m| m.to_string()).unwrap_or("-".into());
        let flags = r.flags.iter().map(RouteFlag::short).collect::<Vec<_>>().join("");
        table.add_row(vec![
            fam, &r.destination.to_string(), &via, dev, &metric, &flags,
        ]);
    }

    println!("{table}");
}

fn flags_short(r: &RouteEntry) -> String {
    if r.flags.is_empty() { return "-".into(); }
    r.flags.iter().map(RouteFlag::short).collect::<Vec<_>>().join("")
}
