use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysInfo {
    pub hostname: String,
    pub os_type: String,
    pub os_version: String,
    pub edition: String,
    pub codename: String,
    pub bitness: String,
    pub architecture: String,
}

pub fn hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|os| os.into_string().ok())
        .unwrap_or_else(|| "unknown".into())
}

pub fn system_info() -> SysInfo {
    let hostname = hostname();
    let info = os_info::get();
    let os_type = info.os_type().to_string();
    let os_version = info.version().to_string();
    let edition = info
        .edition()
        .unwrap_or_else(|| "unknown".into())
        .to_string();
    let codename = info
        .codename()
        .unwrap_or_else(|| "unknown".into())
        .to_string();
    let bitness = if cfg!(target_pointer_width = "64") {
        "64-bit"
    } else {
        "32-bit"
    }
    .into();
    let architecture = std::env::consts::ARCH.into();

    SysInfo {
        hostname,
        os_type,
        os_version,
        edition,
        codename,
        bitness,
        architecture,
    }
}
