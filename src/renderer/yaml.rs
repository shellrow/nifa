use crate::{model::snapshot::Snapshot, net::sys::SysInfo};
use anyhow::Result;
use netdev::Interface;
use serde::Serialize;

pub fn print_interface_yaml(ifaces: &[Interface]) {
    let yaml = serde_yaml::to_string(ifaces).unwrap();
    println!("{}", yaml);
}

pub fn print_snapshot_yaml(sys: &SysInfo, default_iface: Option<Interface>) {
    let snapshot = Snapshot {
        sys: sys.clone(),
        interfaces: default_iface.into_iter().collect(),
    };
    let yaml = serde_yaml::to_string(&snapshot).unwrap();
    println!("{}", yaml);
}

pub fn print_yaml<T: Serialize>(data: &T) -> Result<()> {
    let yaml = serde_yaml::to_string(data)?;
    println!("{}", yaml);
    Ok(())
}
