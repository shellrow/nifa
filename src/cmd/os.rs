use crate::cli::Cli;

/// Show system network stack details
pub fn show_system_net_stack(cli: &Cli) {
    let sys_info = crate::collector::sys::system_info();
    let default_iface_opt = crate::collector::iface::get_default_interface();
    match cli.format {
        crate::cli::OutputFormat::Tree => {
            crate::renderer::tree::print_system_with_default_iface(&sys_info, default_iface_opt)
        }
        crate::cli::OutputFormat::Json => {
            crate::renderer::json::print_snapshot_json(&sys_info, default_iface_opt)
        }
        crate::cli::OutputFormat::Yaml => {
            crate::renderer::yaml::print_snapshot_yaml(&sys_info, default_iface_opt)
        }
    }
}
