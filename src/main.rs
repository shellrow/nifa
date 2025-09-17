use anyhow::Result;
use clap::Parser;
mod cli;
mod collector; 
mod renderer;
mod cmd;

use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

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
        Some(Command::Route) => {

        },
        Some(Command::Os) => {

        },
        Some(Command::Export(_args)) => {

        },
        Some(Command::Monitor(_args)) => {

        },
    };
    Ok(())
}
