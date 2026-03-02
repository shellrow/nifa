use anyhow::Result;
use netsock::{family::AddressFamilyFlags, protocol::ProtocolFlags, socket::ProtocolSocketInfo};

use crate::cli::{SockArgs, SocketFamily, SocketProto};
use crate::model::SockEntry;

pub fn run(args: &SockArgs) -> Result<()> {
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

    let mut socks = crate::net::socket::collect_sockets(pf, af)?;

    if let Some(port) = args.port {
        socks.retain(|s| s.local_port() == port || s.remote_port() == Some(port));
    }

    if args.listen {
        socks.retain(|s| {
            matches!(
                &s.protocol_socket_info,
                ProtocolSocketInfo::Tcp(tcp) if tcp.state.to_string().eq_ignore_ascii_case("listen")
            )
        });
    }

    if args.established {
        socks.retain(|s| {
            matches!(
                &s.protocol_socket_info,
                ProtocolSocketInfo::Tcp(tcp) if tcp.state.to_string().eq_ignore_ascii_case("established")
            )
        });
    }

    let mut entries: Vec<SockEntry> = socks
        .iter()
        .map(|s| {
            let (proto, family, local, remote, state) = match &s.protocol_socket_info {
                ProtocolSocketInfo::Tcp(info) => (
                    "tcp".to_string(),
                    if info.local_addr.is_ipv4() {
                        "ipv4".to_string()
                    } else {
                        "ipv6".to_string()
                    },
                    fmt_sock_addr(info.local_addr, info.local_port),
                    Some(fmt_sock_addr(info.remote_addr, info.remote_port)),
                    Some(info.state.to_string().to_lowercase()),
                ),
                ProtocolSocketInfo::Udp(info) => (
                    "udp".to_string(),
                    if info.local_addr.is_ipv4() {
                        "ipv4".to_string()
                    } else {
                        "ipv6".to_string()
                    },
                    fmt_sock_addr(info.local_addr, info.local_port),
                    None,
                    None,
                ),
            };

            let pid = s.processes.first().map(|p| p.pid);
            let process = s.processes.first().map(|p| p.name.clone());

            SockEntry {
                proto,
                family,
                local,
                remote,
                state,
                pid,
                process,
            }
        })
        .collect();

    entries.sort_by(|a, b| {
        a.proto
            .cmp(&b.proto)
            .then(a.family.cmp(&b.family))
            .then(a.local.cmp(&b.local))
    });

    crate::renderer::render_sockets(&entries, &args.out, args.pid)
}

fn fmt_sock_addr(ip: std::net::IpAddr, port: u16) -> String {
    match ip {
        std::net::IpAddr::V4(v4) => format!("{v4}:{port}"),
        std::net::IpAddr::V6(v6) => format!("[{v6}]:{port}"),
    }
}
