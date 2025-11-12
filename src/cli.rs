use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::cmd::monitor::{SortKey, Unit};

/// nifa - Cross-platform CLI tool for network information
#[derive(Debug, Parser)]
#[command(name = "nifa", author, version, about = "nifa - Cross-platform CLI tool for network information", long_about = None)]
pub struct Cli {
    /// Show only default interface
    #[arg(short, long)]
    pub default: bool,

    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,

    /// With vendor info (OUI lookup)
    #[arg(long, default_value_t = false)]
    pub with_vendor: bool,

    /// Subcommand
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Tree,
    Table,
    Json,
    Yaml,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show all interfaces
    List(ListArgs),
    /// Show details for specified interface
    Show(ShowArgs),
    /// Monitor traffic statistics for all interfaces
    Monitor(MonitorArgs),
    /// Show OS/network stack/permission information
    Os,
    /// Show public IP information
    Public(PublicArgs),
    /// Show routing tables (IPv4/IPv6)
    Route(RouteArgs),
    /// Show neighbor table (ARP/NDP)
    Neigh(NeighArgs),
}

/// List command arguments
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by name (supports partial match)
    #[arg(long)]
    pub name_like: Option<String>,
    /// Show UP status interfaces only
    #[arg(long, conflicts_with = "down")]
    pub up: bool,
    /// Show DOWN status interfaces only
    #[arg(long)]
    pub down: bool,
    /// Show physical interfaces only
    #[arg(long, conflicts_with = "virt")]
    pub phy: bool,
    /// Show virtual interfaces only
    #[arg(long)]
    pub virt: bool,
    /// Show interfaces with IPv4 address only
    #[arg(long)]
    pub ipv4: bool,
    /// Show interfaces with IPv6 address only
    #[arg(long)]
    pub ipv6: bool,
    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,
    /// Export data (JSON/YAML only)
    #[arg(long, default_value_t = false)]
    pub export: bool,
    /// Output file for export
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Show command arguments
#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Show details for specified interface
    pub iface: String,
    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,
    /// Export data (JSON/YAML only)
    #[arg(long, default_value_t = false)]
    pub export: bool,
    /// Output file for export
    #[arg(long)]
    pub output: Option<PathBuf>,
}

/// Monitor command arguments
#[derive(Args, Debug)]
pub struct MonitorArgs {
    /// Target interface (default: all)
    #[arg(short, long)]
    pub iface: Option<String>,
    /// Sort key
    #[arg(short='s', long, value_enum, default_value_t=SortKey::Total)]
    pub sort: SortKey,
    /// Monitor interval in seconds
    #[arg(short = 'd', long, default_value = "1")]
    pub interval: u64,
    /// Display unit (bytes or bits)
    #[arg(long, value_enum, default_value_t=Unit::Bytes)]
    pub unit: Unit,
}

#[derive(Args, Debug)]
pub struct PublicArgs {
    /// IPv4 only
    #[arg(long)]
    pub ipv4: bool,
    /// Timeout seconds
    #[arg(long, default_value_t = 3)]
    pub timeout: u64,
    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,
    /// Export data (JSON/YAML only)
    #[arg(long, default_value_t = false)]
    pub export: bool,
    /// Output file for export
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug)]
pub enum RouteFamilyOpt { All, Ipv4, Ipv6 }

#[derive(Args, Debug)]
pub struct RouteArgs {
    /// Family filter
    #[arg(long, value_enum, default_value_t = RouteFamilyOpt::All)]
    pub family: RouteFamilyOpt,
    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,
    /// Export data (JSON/YAML only)
    #[arg(long, default_value_t = false)]
    pub export: bool,
    /// Output file for export
    #[arg(long)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct NeighArgs {
    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,
    /// Export data (JSON/YAML only)
    #[arg(long, default_value_t = false)]
    pub export: bool,
    /// Output file for export
    #[arg(long)]
    pub output: Option<PathBuf>,
}
