use anyhow::Result;
use serde::Serialize;
use crate::{net::sys::SysInfo, model::snapshot::Snapshot};
use netdev::Interface;

pub fn print_interface_json(ifaces: &[Interface]) {
    let json = serde_json::to_string_pretty(ifaces).unwrap();
    println!("{}", json);
}

pub fn print_snapshot_json(sys: &SysInfo, default_iface: Option<Interface>) {
    let snapshot = Snapshot {
        sys: sys.clone(),
        interfaces: default_iface.into_iter().collect(),
    };
    let json = serde_json::to_string_pretty(&snapshot).unwrap();
    println!("{}", json);
}

pub fn pretty_print_json<T: Serialize>(
    data: &T,
) -> Result<()> {
    let json = serde_json::to_string_pretty(data)?;
    println!("{}", json);
    Ok(())
}
