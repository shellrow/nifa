use anyhow::Result;
use crate::cli::{Cli, OutputFormat, NeighArgs};
use crate::net::neigh;
use crate::renderer::table::make_table;
use termtree::Tree;
use crate::renderer::tree::tree_label;

pub fn run_neigh(_cli: &Cli, args: &NeighArgs) -> Result<()> {
    let table = neigh::get_neighbor_table()?; // HashMap<IpAddr, MacAddr>

    if args.export {
        crate::fs::export(
            args.format,
            args.output.as_deref(),
            &table,
        ).unwrap_or_else(|e| {
            tracing::error!("Export failed: {}", e);
        });
    } else {
        match args.format {
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&table)?),
            OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&table)?),
            OutputFormat::Table => {
                print_neigh_table(&table);
            }
            OutputFormat::Tree => {
                print_neigh_tree(&table);
            }
        }
    }
    
    Ok(())
}

fn print_neigh_tree(table: &std::collections::HashMap<std::net::IpAddr, netdev::MacAddr>) {
    let host = crate::net::sys::hostname();
    let mut root = Tree::new(tree_label(format!("Neighbors (ARP/NDP) on {}", host)));

    let mut v4 = Tree::new(tree_label("IPv4"));
    let mut v6 = Tree::new(tree_label("IPv6"));

    let mut keys: Vec<_> = table.keys().cloned().collect();
    keys.sort_by(|a,b| a.to_string().cmp(&b.to_string()));

    for ip in keys {
        let mac = table.get(&ip).unwrap();
        let leaf = Tree::new(format!("{}  ->  {}", ip, mac));
        match ip {
            std::net::IpAddr::V4(_) => {
                v4.push(leaf);
            },
            std::net::IpAddr::V6(_) => {
                v6.push(leaf);
            },
        }
    }

    if !v4.leaves.is_empty() { root.push(v4); }
    if !v6.leaves.is_empty() { root.push(v6); }
    println!("{}", root);
}

fn print_neigh_table(table: &std::collections::HashMap<std::net::IpAddr, netdev::MacAddr>) {
    let mut tbl = make_table(&["IP ADDRESS", "MAC ADDRESS"]);

    let mut rows: Vec<_> = table.iter().collect();
    rows.sort_by(|(a, _), (b, _)| a.to_string().cmp(&b.to_string()));

    for (ip, mac) in rows {
        tbl.add_row(vec![ip.to_string(), mac.to_string()]);
    }

    println!("{tbl}");
}
