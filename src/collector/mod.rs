pub mod iface;
pub mod sys;

use anyhow::Result;

use crate::model::snapshot::Snapshot;

pub fn collect_snapshot() -> Result<Snapshot> {
    let sys = crate::collector::sys::system_info();
    let interfaces = crate::collector::iface::collect_all_interfaces();
    Ok(Snapshot { sys, interfaces })
}
