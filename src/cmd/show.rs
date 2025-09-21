use crate::cli::Cli;
use crate::cli::ShowArgs;
use crate::collector;
use crate::renderer;

/// Show specified interface details
pub fn show_interface(cli: &Cli, args: &ShowArgs) {
    match collector::iface::get_interface_by_name(&args.iface) {
        Some(iface) => {
            // Render output
            match cli.format {
                crate::cli::OutputFormat::Tree => renderer::tree::print_interface_detail_tree(&iface),
                crate::cli::OutputFormat::Json => renderer::json::print_interface_json(&[iface]),
                crate::cli::OutputFormat::Yaml => renderer::yaml::print_interface_yaml(&[iface]),
            }
        },
        None => {
            tracing::error!("Interface '{}' not found", args.iface);
        }
    }    
}
