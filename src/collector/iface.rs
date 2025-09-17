use netdev::Interface;

pub fn collect_all_interfaces() -> Vec<Interface> {
    netdev::get_interfaces()
}

pub fn get_default_interface() -> Option<Interface> {
    match netdev::get_default_interface() {
        Ok(iface) => Some(iface),
        Err(_) => None,
    }
}

pub fn get_interface_by_name(name: &str) -> Option<Interface> {
    let interfaces = netdev::get_interfaces();
    for iface in interfaces {
        if iface.name == name {
            return Some(iface);
        }
    }
    None
}
