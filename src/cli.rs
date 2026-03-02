use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::cmd::monitor::{SortKey, Unit};

/// nifa - Cross-platform network inspection tool
#[derive(Debug, Parser)]
#[command(
    name = "nifa",
    author,
    version,
    about = "Cross-platform network inspection tool"
)]
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
    Auto,
    Tree,
    Table,
    Json,
    Yaml,
}

#[derive(Args, Debug, Clone)]
pub struct OutputArgs {
    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Auto)]
    pub format: OutputFormat,
    /// Prefer wider table layout
    #[arg(long, default_value_t = false)]
    pub wide: bool,
    /// Disable colored output
    #[arg(long, default_value_t = false)]
    pub no_color: bool,
    /// Disable truncation in compact output
    #[arg(long, default_value_t = false)]
    pub no_truncate: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Network interfaces
    #[command(name = "if")]
    If(IfArgs),
    /// IP addresses
    Addr(AddrArgs),
    /// Layer 2 information
    Link(LinkArgs),
    /// Routing table
    Route(RouteArgs),
    /// ARP/NDP entries
    Neigh(NeighArgs),
    /// Sockets (ss/netstat equivalent)
    Sock(SockArgs),
    /// Network/system summary
    Sys(SysArgs),
    /// TUI monitor
    Mon(MonArgs),
}

#[derive(Debug, Subcommand)]
pub enum ShowAction {
    /// Show details for a specific target
    Show { target: String },
}

#[derive(Args, Debug)]
pub struct IfArgs {
    #[command(subcommand)]
    pub action: Option<ShowAction>,
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
    /// Resolve vendor info using OUI DB
    #[arg(long, default_value_t = false)]
    pub vendor: bool,
    #[command(flatten)]
    pub out: OutputArgs,
}

#[derive(Args, Debug)]
pub struct AddrArgs {
    #[command(subcommand)]
    pub action: Option<ShowAction>,
    /// Filter by interface name
    #[arg(long)]
    pub iface: Option<String>,
    /// Show IPv4 only
    #[arg(long, conflicts_with = "ipv6")]
    pub ipv4: bool,
    /// Show IPv6 only
    #[arg(long)]
    pub ipv6: bool,
    #[command(flatten)]
    pub out: OutputArgs,
}

#[derive(Args, Debug)]
pub struct LinkArgs {
    /// Filter by interface name
    #[arg(long)]
    pub iface: Option<String>,
    /// Show UP status interfaces only
    #[arg(long, conflicts_with = "down")]
    pub up: bool,
    /// Show DOWN status interfaces only
    #[arg(long)]
    pub down: bool,
    #[command(flatten)]
    pub out: OutputArgs,
}

#[derive(Args, Debug)]
pub struct RouteArgs {
    /// Show IPv4 routes only
    #[arg(long, conflicts_with = "ipv6")]
    pub ipv4: bool,
    /// Show IPv6 routes only
    #[arg(long)]
    pub ipv6: bool,
    /// Show default routes only
    #[arg(long)]
    pub default: bool,
    /// Show detailed route metadata
    #[arg(long, default_value_t = false)]
    pub detail: bool,
    #[command(flatten)]
    pub out: OutputArgs,
}

#[derive(Args, Debug)]
pub struct NeighArgs {
    /// Filter by interface name (best-effort by platform support)
    #[arg(long)]
    pub iface: Option<String>,
    /// Show IPv4 only
    #[arg(long, conflicts_with = "ipv6")]
    pub ipv4: bool,
    /// Show IPv6 only
    #[arg(long)]
    pub ipv6: bool,
    /// Resolve vendor info using OUI DB
    #[arg(long, default_value_t = false)]
    pub vendor: bool,
    #[command(flatten)]
    pub out: OutputArgs,
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
pub struct SockArgs {
    /// Protocol filter
    #[arg(long, value_enum, default_value = "all")]
    pub proto: SocketProto,
    /// Address family filter
    #[arg(long, value_enum, default_value = "all")]
    pub family: SocketFamily,
    /// Show listening sockets only
    #[arg(long, conflicts_with = "established")]
    pub listen: bool,
    /// Show established sockets only
    #[arg(long)]
    pub established: bool,
    /// Filter by local or remote port
    #[arg(long)]
    pub port: Option<u16>,
    /// Include process ownership details
    #[arg(long, default_value_t = false)]
    pub pid: bool,
    #[command(flatten)]
    pub out: OutputArgs,
}

#[derive(Args, Debug)]
pub struct SysArgs {
    /// Emphasize DNS details
    #[arg(long, default_value_t = false)]
    pub dns: bool,
    /// Emphasize proxy details
    #[arg(long, default_value_t = false)]
    pub proxy: bool,
    /// Show summary-only output
    #[arg(long, default_value_t = false)]
    pub summary: bool,
    #[command(flatten)]
    pub out: OutputArgs,
}

#[derive(Args, Debug)]
pub struct MonArgs {
    /// Target interface (default: all)
    #[arg(long)]
    pub iface: Option<String>,
    /// Monitor interval in seconds
    #[arg(long, default_value = "1")]
    pub interval: u64,
    /// Sort key
    #[arg(long, value_enum, default_value_t = SortKey::Total)]
    pub sort: SortKey,
    /// Display unit (bytes or bits)
    #[arg(long, value_enum, default_value_t = Unit::default())]
    pub unit: Unit,
}
