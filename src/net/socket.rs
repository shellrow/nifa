use anyhow::Result;
use netsock::{
    family::AddressFamilyFlags, get_sockets, protocol::ProtocolFlags, socket::SocketInfo,
};

pub fn collect_sockets(
    proto: ProtocolFlags,
    family: AddressFamilyFlags,
) -> Result<Vec<SocketInfo>> {
    get_sockets(family, proto).map_err(|e| anyhow::anyhow!("Failed to collect sockets: {}", e))
}
