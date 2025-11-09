pub mod iface;
pub mod sys;

use anyhow::Result;

use crate::model::snapshot::Snapshot;

pub fn collect_snapshot() -> Result<Snapshot> {
    let sys = crate::net::sys::system_info();
    let interfaces = crate::net::iface::collect_all_interfaces();
    Ok(Snapshot { sys, interfaces })
}
