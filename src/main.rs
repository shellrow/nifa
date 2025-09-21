use anyhow::Result;
use clap::Parser;
mod cli;
mod collector; 
mod renderer;
mod cmd;
mod model;
mod db;

use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.with_vendor {
        db::oui::init_oui_db()?;
    }

    match &cli.command {
        None => {
            cmd::list::show_interfaces(&cli);
        },
        Some(Command::List(args)) => {
            cmd::list::list_interfaces(&cli, args);
        },
        Some(Command::Show(args)) => {
            cmd::show::show_interface(&cli, args);
        },
        Some(Command::Os) => {
            cmd::os::show_system_net_stack(&cli);
        },
        Some(Command::Export(args)) => {
            cmd::export::export_snapshot(&cli, args)?;
        },
        Some(Command::Monitor(args)) => {
            cmd::monitor::monitor_interfaces(&cli, args)?;
        },
        Some(Command::Public(args)) => {
            cmd::public::show_public_ip_info(&cli, args).await?;
        }
    };
    Ok(())
}
