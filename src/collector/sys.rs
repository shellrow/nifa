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
    pub proxy: ProxyEnv,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyEnv {
    pub http: Option<String>,
    pub https: Option<String>,
    pub all: Option<String>,
    pub no_proxy: Option<String>,
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

    let proxy = collect_proxy_env();

    SysInfo {
        hostname,
        os_type,
        os_version,
        edition,
        codename,
        bitness,
        architecture,
        proxy,
    }
}

/// Collect proxy environment variables
pub fn collect_proxy_env() -> ProxyEnv {
    // Prefer lowercase, fallback to uppercase
    fn pick(key: &str) -> Option<String> {
        std::env::var(key.to_lowercase())
            .ok()
            .or_else(|| std::env::var(key.to_uppercase()).ok())
            .filter(|s| !s.trim().is_empty())
    }

    ProxyEnv {
        http: pick("http_proxy"),
        https: pick("https_proxy"),
        all: pick("all_proxy"),
        no_proxy: pick("no_proxy"),
    }
}
