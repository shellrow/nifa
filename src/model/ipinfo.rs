use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IpInfo {
    pub ip_version: String,
    pub ip_addr_dec: String,
    pub ip_addr: String,
    pub host_name: String,
    pub network: String,
    pub asn: String,
    pub as_name: String,
    pub country_code: String,
    pub country_name: String,
}

#[derive(Debug, Serialize)]
pub struct PublicOut {
    pub common: Option<CommonInfo>,
    pub ipv4: Option<IpSide>,
    pub ipv6: Option<IpSide>,
}

#[derive(Debug, Serialize)]
pub struct CommonInfo {
    pub asn: String,
    pub as_name: String,
    pub country_code: String,
    pub country_name: String,
}

#[derive(Debug, Serialize)]
pub struct IpSide {
    pub ip_addr: String,
    pub ip_addr_dec: String,
    pub host_name: String,
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_name: Option<String>,
}
