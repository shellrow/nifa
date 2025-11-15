use std::net::IpAddr;
use comfy_table::{Row, Cell};
use termtree::Tree;
use anyhow::Result;
use netsock::socket::ProtocolSocketInfo;
use crate::cli::{Cli, OutputFormat, SocketArgs, SocketFamily, SocketProto};
use crate::fs::export;
use crate::net::addr::AddressFamily;
use crate::net::socket::collect_sockets;
use crate::renderer::tree::tree_label;

use netsock::{
    protocol::ProtocolFlags,
    family::AddressFamilyFlags,
    socket::SocketInfo,
};

pub fn show_sockets(_cli: &Cli, args: &SocketArgs) -> Result<()> {
    let pf = match args.proto {
        SocketProto::Tcp => ProtocolFlags::TCP,
        SocketProto::Udp => ProtocolFlags::UDP,
        SocketProto::All => ProtocolFlags::TCP | ProtocolFlags::UDP,
    };

    let af = match args.family {
        SocketFamily::Ipv4 => AddressFamilyFlags::IPV4,
        SocketFamily::Ipv6 => AddressFamilyFlags::IPV6,
        SocketFamily::All => AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6,
    };

    let mut socks = collect_sockets(pf, af)?;

    if let Some(pid) = args.pid {
        socks.retain(|s| s.is_owned_by_pid(pid));
    }
    if let Some(port) = args.port {
        socks.retain(|s| {
            s.local_port() == port || s.remote_port() == Some(port)
        });
    }
    if let Some(ref state) = args.state {
        let want = state.to_lowercase();
        socks.retain(|s| {
            match &s.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp_sock) => {
                    let sock_state = tcp_sock.state.to_string().to_lowercase();
                    want == "all" || sock_state == want
                }
                _ => false,
            }
        });
    }

    if args.export {
        export(args.format, args.output.as_deref(), &socks)?;
    }

    match args.format {
        OutputFormat::Tree => print_socket_tree(&socks),
        OutputFormat::Table => print_socket_table(&socks),
        OutputFormat::Json => crate::renderer::json::pretty_print_json(&socks)?,
        OutputFormat::Yaml => crate::renderer::yaml::print_yaml(&socks)?,
    }

    Ok(())
}

fn fmt_sock_addr(ip: IpAddr, port: u16) -> String {
    match ip {
        IpAddr::V4(v4) => format!("{}:{}", v4, port),
        IpAddr::V6(v6) => format!("[{}]:{}", v6, port),
    }
}

fn fmt_processes(s: &SocketInfo) -> (String, String) {
    if s.processes.is_empty() {
        return ("-".to_string(), "-".to_string());
    }

    let first = &s.processes[0];
    let pid = if s.processes.len() == 1 {
        format!("{}", first.pid)
    } else {
        format!("{} (+{})", first.pid, s.processes.len() - 1)
    };

    let name = if s.processes.len() == 1 {
        first.name.clone()
    } else {
        format!("{} (+{})", first.name, s.processes.len() - 1)
    };

    (pid, name)
}

pub fn print_socket_table(socks: &[SocketInfo]) {
    if socks.is_empty() {
        println!("(no sockets)");
        return;
    }

    let mut table = crate::renderer::table::make_table(&["Proto", "Fam", "Local Address", "Remote Address", "State", "PID", "Process"]);

    for s in socks {
        let fam = AddressFamily::from_ip_addr(&s.local_addr());
        let (proto, fam, local, remote, state) = match &s.protocol_socket_info {
            ProtocolSocketInfo::Tcp(info) => {
                (
                    "TCP".to_string(),
                    fam.to_string(),
                    fmt_sock_addr(info.local_addr, info.local_port),
                    fmt_sock_addr(info.remote_addr, info.remote_port),
                    format!("{:?}", info.state),
                )
            }
            ProtocolSocketInfo::Udp(info) => {
                (
                    "UDP".to_string(),
                    fam.to_string(),
                    fmt_sock_addr(info.local_addr, info.local_port),
                    "-".to_string(),
                    "-".to_string(),
                )
            }
        };

        let (pid, pname) = fmt_processes(s);

        table.add_row(Row::from(vec![
            Cell::new(proto),
            Cell::new(fam),
            Cell::new(local),
            Cell::new(remote),
            Cell::new(state),
            Cell::new(pid),
            Cell::new(pname),
        ]));
    }

    println!("{table}");
}

pub fn print_socket_tree(socks: &[SocketInfo]) {
    let host = crate::net::sys::hostname();
    let mut root = Tree::new(tree_label(format!("Sockets on {}", host)));

    if socks.is_empty() {
        root.push(Tree::new(tree_label("(no sockets)")));
        println!("{root}");
        return;
    }

    for s in socks {
        let mut node = match &s.protocol_socket_info {
            ProtocolSocketInfo::Tcp(info) => {
                let fam = match info.local_addr {
                    IpAddr::V4(_) => "IPv4",
                    IpAddr::V6(_) => "IPv6",
                };
                let title = format!(
                    "TCP [{}] {} -> {} ({:?})",
                    fam,
                    fmt_sock_addr(info.local_addr, info.local_port),
                    fmt_sock_addr(info.remote_addr, info.remote_port),
                    info.state,
                );
                Tree::new(tree_label(title))
            }
            ProtocolSocketInfo::Udp(info) => {
                let fam = match info.local_addr {
                    IpAddr::V4(_) => "IPv4",
                    IpAddr::V6(_) => "IPv6",
                };
                let title = format!(
                    "UDP [{}] {}",
                    fam,
                    fmt_sock_addr(info.local_addr, info.local_port),
                );
                Tree::new(tree_label(title))
            }
        };

        // Processes
        if s.processes.is_empty() {
            node.push(Tree::new(tree_label("Process: (unknown)")));
        } else {
            let mut procs = Tree::new(tree_label("Processes"));
            for p in &s.processes {
                procs.push(Tree::new(tree_label(format!("{} ({})", p.pid, p.name))));
            }
            node.push(procs);
        }

        #[cfg(target_os = "linux")]
        {
            node.push(Tree::new(tree_label(format!("UID: {}", s.uid))));
            node.push(Tree::new(tree_label(format!("Inode: {}", s.inode))));
        }
        root.push(node);
    }

    println!("{root}");
}
