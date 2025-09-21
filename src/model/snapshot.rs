use netdev::Interface;
use serde::{Serialize, Deserialize};

use crate::collector::sys::SysInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub sys: SysInfo,
    pub interfaces: Vec<Interface>,
}
