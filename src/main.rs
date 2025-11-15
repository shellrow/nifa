use anyhow::Result;
use clap::Parser;
mod cli;
mod cmd;
mod net;
mod db;
mod model;
mod renderer;
mod fs;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.with_vendor {
        db::oui::init_oui_db()?;
    }

    match &cli.command {
        None => {
            cmd::ifaces::show_interfaces(&cli);
        }
        Some(Command::Ifaces(args)) => {
            cmd::ifaces::list_interfaces(&cli, args);
        }
        Some(Command::Iface(args)) => {
            cmd::iface::show_interface(&cli, args);
        }
        Some(Command::System) => {
            cmd::system::show_system_net_stack(&cli);
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
