use anyhow::Result;
use clap::Parser;
mod cli;
mod cmd;
mod db;
mod fs;
mod log;
mod model;
mod net;
mod renderer;
mod time;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    log::init_logger(&cli.log_level)?;

    match &cli.command {
        None => {
            cmd::iface::show_default_interface(&cli)?;
        }
        Some(Command::Ifaces(args)) => {
            cmd::ifaces::list_interfaces(&cli, args)?;
        }
        Some(Command::Iface(args)) => {
            cmd::iface::show_interface(&cli, args)?;
        }
        Some(Command::System(args)) => {
            cmd::system::show_system_net_stack(&cli, args)?;
        }
        Some(Command::Monitor(args)) => {
            cmd::monitor::monitor_interfaces(&cli, args)?;
        }
        Some(Command::Public(args)) => {
            cmd::public::show_public_ip_info(&cli, args).await?;
        }
        Some(Command::Route(args)) => {
            cmd::route::show_route(&cli, args)?;
        }
        Some(Command::Neigh(args)) => {
            cmd::neigh::show_neigh(&cli, args)?;
        }
        Some(Command::Socket(args)) => {
            cmd::socket::show_sockets(&cli, args)?;
        }
    };
    Ok(())
}
