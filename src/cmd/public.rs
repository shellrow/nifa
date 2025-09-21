use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;

use crate::cli::{Cli, OutputFormat, PublicArgs};
use crate::model::ipinfo::{CommonInfo, IpInfo, IpSide, PublicOut};
use crate::renderer::tree::print_public_ip_tree;

const IPSTRUCT_URL: &str = "https://api.ipstruct.com/ip";
const IPSTRUCT_V4_URL: &str = "https://ipv4.ipstruct.com/ip";
//const IP_VERSION_4: &str = "v4";
const IP_VERSION_6: &str = "v6";

/// Show public IP information
pub async fn show_public_ip_info(cli: &Cli, args: &PublicArgs) -> Result<()> {
    let client = Client::builder()
        .timeout(Duration::from_secs(args.timeout.max(1)))
        .build()
        .context("build http client")?;

    let v4: Option<IpInfo>;
    let mut v6: Option<IpInfo> = None;

    if args.v4_only {
        v4 = fetch_ip(&client, IPSTRUCT_V4_URL).await?;
    } else {
        let (any_res, v4_res) = tokio::join!(
            fetch_ip(&client, IPSTRUCT_URL),
            fetch_ip(&client, IPSTRUCT_V4_URL),
        );

        let any = any_res.unwrap_or(None);
        let v4opt = v4_res.unwrap_or(None);

        match any {
            Some(info) if is_ipv6(&info) => {
                v6 = Some(info);
                v4 = v4opt;
            }
            Some(info) => {
                v4 = Some(info);
            }
            None => {
                v4 = v4opt;
            }
        }
    }

    let out = build_public_out(v4, v6);

    match cli.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&out)?),
        OutputFormat::Yaml => println!("{}", serde_yaml::to_string(&out)?),
        _ => print_public_ip_tree(&out),
    }
    Ok(())
}

/// Fetch IP information from a given URL
async fn fetch_ip(client: &Client, url: &str) -> Result<Option<IpInfo>> {
    let resp = client.get(url).send().await.with_context(|| format!("GET {}", url))?;
    if !resp.status().is_success() {
        anyhow::bail!("{} -> HTTP {}", url, resp.status());
    }
    let info: IpInfo = resp.json().await.context("parse json IpInfo")?;
    Ok(Some(info))
}

fn is_ipv6(info: &IpInfo) -> bool {
    info.ip_version == IP_VERSION_6 || info.ip_addr.contains(':')
}

fn build_public_out(v4: Option<IpInfo>, v6: Option<IpInfo>) -> PublicOut {
    // v4 or v6 is missing, cannot commonize
    if v4.is_none() || v6.is_none() {
        return PublicOut {
            common: None,
            ipv4: v4.as_ref().map(|i| IpSide {
                ip_addr: i.ip_addr.clone(),
                ip_addr_dec: i.ip_addr_dec.clone(),
                host_name: i.host_name.clone(),
                network: i.network.clone(),
                asn: Some(i.asn.clone()),
                as_name: Some(i.as_name.clone()),
                country_code: Some(i.country_code.clone()),
                country_name: Some(i.country_name.clone()),
            }),
            ipv6: v6.as_ref().map(|i| IpSide {
                ip_addr: i.ip_addr.clone(),
                ip_addr_dec: i.ip_addr_dec.clone(),
                host_name: i.host_name.clone(),
                network: i.network.clone(),
                asn: Some(i.asn.clone()),
                as_name: Some(i.as_name.clone()),
                country_code: Some(i.country_code.clone()),
                country_name: Some(i.country_name.clone()),
            }),
        };
    }

    let v4i = v4.as_ref().unwrap();
    let v6i = v6.as_ref().unwrap();

    let same_asn        = v4i.asn == v6i.asn;
    let same_as_name    = v4i.as_name == v6i.as_name;
    let same_cc         = v4i.country_code == v6i.country_code;
    let same_country    = v4i.country_name == v6i.country_name;

    // If all fields are the same, we can commonize
    if same_asn && same_as_name && same_cc && same_country {
        PublicOut {
            common: Some(CommonInfo {
                asn: v4i.asn.clone(),
                as_name: v4i.as_name.clone(),
                country_code: v4i.country_code.clone(),
                country_name: v4i.country_name.clone(),
            }),
            ipv4: Some(IpSide {
                ip_addr: v4i.ip_addr.clone(),
                ip_addr_dec: v4i.ip_addr_dec.clone(),
                host_name: v4i.host_name.clone(),
                network: v4i.network.clone(),
                asn: None, as_name: None, country_code: None, country_name: None,
            }),
            ipv6: Some(IpSide {
                ip_addr: v6i.ip_addr.clone(),
                ip_addr_dec: v6i.ip_addr_dec.clone(),
                host_name: v6i.host_name.clone(),
                network: v6i.network.clone(),
                asn: None, as_name: None, country_code: None, country_name: None,
            }),
        }
    } else {
        PublicOut {
            common: None,
            ipv4: Some(IpSide {
                ip_addr: v4i.ip_addr.clone(),
                ip_addr_dec: v4i.ip_addr_dec.clone(),
                host_name: v4i.host_name.clone(),
                network: v4i.network.clone(),
                asn: Some(v4i.asn.clone()),
                as_name: Some(v4i.as_name.clone()),
                country_code: Some(v4i.country_code.clone()),
                country_name: Some(v4i.country_name.clone()),
            }),
            ipv6: Some(IpSide {
                ip_addr: v6i.ip_addr.clone(),
                ip_addr_dec: v6i.ip_addr_dec.clone(),
                host_name: v6i.host_name.clone(),
                network: v6i.network.clone(),
                asn: Some(v6i.asn.clone()),
                as_name: Some(v6i.as_name.clone()),
                country_code: Some(v6i.country_code.clone()),
                country_name: Some(v6i.country_name.clone()),
            }),
        }
    }
}
