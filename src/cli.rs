use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::cmd::monitor::{SortKey, Unit};

/// nifa - Cross-platform CLI tool for network information
#[derive(Debug, Parser)]
#[command(name = "nifa", author, version, about = "nifa - Cross-platform CLI tool for network information", long_about = None)]
pub struct Cli {
    /// Set log level
    #[arg(short = 'l', long, value_enum, default_value_t = LogLevel::Error)]
    pub log_level: LogLevel,
    /// Subcommand
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_tracing_level(&self) -> tracing::Level {
        match self {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Tree,
    Table,
    Json,
    Yaml,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    Json,
    Yaml,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Show all interfaces
    Ifaces(IfacesArgs),
    /// Show details for specified interface
    Iface(IfaceArgs),
    /// Monitor traffic statistics for interfaces in TUI
    Monitor(MonitorArgs),
    /// Show routing tables (IPv4/IPv6)
    Route(RouteArgs),
    /// Show neighbor table (ARP/NDP)
    Neigh(NeighArgs),
    /// Show open TCP/UDP sockets and associated processes
    Socket(SocketArgs),
    /// Show public IP information
    Public(PublicArgs),
    /// Show OS / kernel / proxy / default interface
    System(SystemArgs),
}

/// Ifaces command arguments
#[derive(Args, Debug)]
pub struct IfacesArgs {
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
    /// Export data instead of printing to stdout
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,
    /// Output file for export
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
    /// With vendor info (OUI lookup)
    #[arg(long, default_value_t = false)]
    pub vendor: bool,
}

/// Iface command arguments
#[derive(Args, Debug)]
pub struct IfaceArgs {
    /// Show details for specified interface
    pub iface: String,
    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,
    /// Export data instead of printing to stdout
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,
    /// Output file for export
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
    /// With vendor info (OUI lookup)
    #[arg(long, default_value_t = false)]
    pub vendor: bool,
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
    #[arg(long, value_enum, default_value_t=Unit::default())]
    pub unit: Unit,
}

/// System command arguments
#[derive(Args, Debug)]
pub struct SystemArgs {
    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,
    /// Export data instead of printing to stdout
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,
    /// Output file for export
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
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
    /// Export data instead of printing to stdout
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,
    /// Output file for export
    #[arg(short = 'o', long)]
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
    /// Export data instead of printing to stdout
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,
    /// Output file for export
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct NeighArgs {
    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,
    /// Export data instead of printing to stdout
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,
    /// Output file for export
    #[arg(short = 'o', long)]
    pub output: Option<PathBuf>,
    /// With vendor info (OUI lookup)
    #[arg(long, default_value_t = false)]
    pub vendor: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SocketProto {
    Tcp,
    Udp,
    All,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SocketFamily {
    Ipv4,
    Ipv6,
    All,
}

#[derive(Args, Debug)]
pub struct SocketArgs {
    /// Protocol filter
    #[arg(long, value_enum, default_value = "all")]
    pub proto: SocketProto,

    /// Address family filter
    #[arg(long, value_enum, default_value = "all")]
    pub family: SocketFamily,

    /// TCP state filter (established, listen, time_wait, all)
    #[arg(long)]
    pub state: Option<String>,

    /// Filter by local or remote port
    #[arg(long)]
    pub port: Option<u16>,

    /// Filter by PID
    #[arg(long)]
    pub pid: Option<u32>,

    /// Output format
    #[arg(short='f', long, value_enum, default_value_t = OutputFormat::Tree)]
    pub format: OutputFormat,

    /// Export data instead of printing to stdout
    #[arg(long, value_enum)]
    pub export: Option<ExportFormat>,

    /// Output file for export
    #[arg(short = 'o', long)]
    pub output: Option<std::path::PathBuf>,
}
