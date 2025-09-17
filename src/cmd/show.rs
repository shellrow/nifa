use netdev::Interface;
use crate::cli::Cli;
use crate::cli::ShowArgs;
use crate::collector;
use crate::renderer;

/// Default action with no subcommand
pub fn show_interface(cli: &Cli, args: &ShowArgs) {
    let interfaces: Vec<Interface> = collector::iface::get_interface_by_name(&args.iface).into_iter().collect();
    // Render output
    match cli.format {
        crate::cli::OutputFormat::Tree => renderer::tree::print_interface_tree(&interfaces),
        //crate::cli::OutputFormat::Table => renderer::table::print_interface_table(&interfaces),
        //crate::cli::OutputFormat::Json => renderer::json::print_interface_json(&interfaces),
        //crate::cli::OutputFormat::Yaml => renderer::yaml::print_interface_yaml(&interfaces),
        _ => unimplemented!("Currently only tree format is implemented for show command"),
    }
}
