use netdev::Interface;
use crate::cli::Cli;
use crate::cli::ListArgs;
use crate::collector;
use crate::renderer;

/// Default action with no subcommand
pub fn show_interfaces(cli: &Cli) {
    let interfaces: Vec<Interface> = if cli.default {
        collector::iface::get_default_interface().into_iter().collect()
    } else {
        collector::iface::collect_all_interfaces()
    };
    // Render output
    match cli.format {
        crate::cli::OutputFormat::Tree => renderer::tree::print_interface_tree(&interfaces),
        crate::cli::OutputFormat::Json => renderer::json::print_interface_json(&interfaces),
        crate::cli::OutputFormat::Yaml => renderer::yaml::print_interface_yaml(&interfaces),
    }
}

pub fn list_interfaces(cli: &Cli, args: &ListArgs) {
    let mut interfaces: Vec<Interface> = collector::iface::collect_all_interfaces();

    // Apply filters
    if let Some(name_like) = &args.name_like {
        interfaces.retain(|iface| iface.name.contains(name_like));
    }
    if args.up {
        interfaces.retain(|iface| iface.oper_state == netdev::interface::OperState::Up);
    }
    if args.down {
        interfaces.retain(|iface| iface.oper_state == netdev::interface::OperState::Down);
    }
    if args.physical {
        interfaces.retain(|iface| iface.is_physical());
    }
    if args.virt {
        interfaces.retain(|iface| !iface.is_physical());
    }
    if args.has_ipv4 {
        interfaces.retain(|iface| !iface.ipv4.is_empty());
    }
    if args.has_ipv6 {
        interfaces.retain(|iface| !iface.ipv6.is_empty());
    }

    // Render output
    match cli.format {
        crate::cli::OutputFormat::Tree => renderer::tree::print_interface_tree(&interfaces),
        crate::cli::OutputFormat::Json => renderer::json::print_interface_json(&interfaces),
        crate::cli::OutputFormat::Yaml => renderer::yaml::print_interface_yaml(&interfaces),
    }
}
