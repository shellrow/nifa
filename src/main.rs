use anyhow::Result;
use clap::Parser;

mod cli;
mod cmd;
mod db;
mod log;
mod model;
mod net;
mod renderer;
mod time;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();
    log::init_logger(&cli.log_level)?;

    match &cli.command {
        Some(Command::If(args)) => cmd::ifc::run(args)?,
        Some(Command::Addr(args)) => cmd::addr::run(args)?,
        Some(Command::Link(args)) => cmd::link::run(args)?,
        Some(Command::Route(args)) => cmd::route::run(args)?,
        Some(Command::Neigh(args)) => cmd::neigh::run(args)?,
        Some(Command::Sock(args)) => cmd::sock::run(args)?,
        Some(Command::Sys(args)) => cmd::sys::run(args)?,
        Some(Command::Mon(args)) => cmd::monitor::monitor_interfaces(args)?,
        None => {
            cmd::ifc::run_default_interface()?;
        }
    }

    Ok(())
}
