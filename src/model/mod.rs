use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfSummary {
    pub index: u32,
    pub name: String,
    pub if_type: String,
    pub state: String,
    pub is_default: bool,
    pub mac: Option<String>,
    pub mtu: Option<u32>,
    pub ipv4_addrs: Vec<String>,
    pub ipv6_addrs: Vec<String>,
    pub gateway_mac: Option<String>,
    pub gateway_ipv4: Vec<String>,
    pub gateway_ipv6: Vec<String>,
    pub vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfDetail {
    pub summary: IfSummary,
    pub friendly_name: Option<String>,
    pub description: Option<String>,
    pub flags: String,
    pub tx_speed: Option<String>,
    pub rx_speed: Option<String>,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub ipv6_scoped: Vec<String>,
    pub dns_servers: Vec<String>,
    pub gateway_ipv4: Vec<String>,
    pub gateway_ipv6: Vec<String>,
    pub gateway_mac: Option<String>,
    pub stats_rx_bytes: Option<u64>,
    pub stats_tx_bytes: Option<u64>,
    pub vpn_like: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAddrEntry {
    pub iface: String,
    pub if_type: String,
    pub family: String,
    pub address: String,
    pub prefix_len: u8,
    pub scope_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub family: String,
    pub destination: String,
    pub gateway: Option<String>,
    pub interface: Option<String>,
    pub metric: Option<u32>,
    pub flags: Vec<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeighEntry {
    pub iface: Option<String>,
    pub family: String,
    pub ip: String,
    pub mac: String,
    pub vendor: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SockEntry {
    pub proto: String,
    pub family: String,
    pub local: String,
    pub remote: Option<String>,
    pub state: Option<String>,
    pub pid: Option<u32>,
    pub process: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSummary {
    pub hostname: String,
    pub os: String,
    pub kernel_version: Option<String>,
    pub default_interface: Option<String>,
    pub default_gateway: Option<String>,
    pub dns_servers: Vec<String>,
    pub proxy_http: Option<String>,
    pub proxy_https: Option<String>,
    pub proxy_all: Option<String>,
    pub proxy_no_proxy: Option<String>,
}
