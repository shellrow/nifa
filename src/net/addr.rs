use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub enum AddressFamily {
    Ipv4,
    Ipv6,
}

impl AddressFamily {
    pub fn from_ip_addr(ip: &IpAddr) -> Self {
        match ip {
            IpAddr::V4(_) => AddressFamily::Ipv4,
            IpAddr::V6(_) => AddressFamily::Ipv6,
        }
    }

    #[allow(dead_code)]
    pub fn from_socket_addr(sock: &SocketAddr) -> Self {
        match sock {
            SocketAddr::V4(_) => AddressFamily::Ipv4,
            SocketAddr::V6(_) => AddressFamily::Ipv6,
        }
    }

    pub fn unspecified_ip(&self) -> IpAddr {
        match self {
            AddressFamily::Ipv4 => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            AddressFamily::Ipv6 => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
        }
    }

    pub fn unspecified_sock(&self) -> SocketAddr {
        SocketAddr::new(self.unspecified_ip(), 0)
    }
}

impl ToString for AddressFamily {
    fn to_string(&self) -> String {
        match self {
            AddressFamily::Ipv4 => "IPv4".to_string(),
            AddressFamily::Ipv6 => "IPv6".to_string(),
        }
    }
}

#[allow(dead_code)]
pub fn unwrap_or_unspecified_ip(ip: Option<IpAddr>, family: AddressFamily) -> IpAddr {
    ip.unwrap_or_else(|| family.unspecified_ip())
}

#[allow(dead_code)]
pub fn unwrap_or_unspecified_sock(ip: Option<SocketAddr>, family: AddressFamily) -> SocketAddr {
    ip.unwrap_or_else(|| family.unspecified_sock())
}
