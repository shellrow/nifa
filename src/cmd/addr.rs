use anyhow::Result;

use crate::cli::{AddrArgs, ShowAction};
use crate::model::IpAddrEntry;

pub fn run(args: &AddrArgs) -> Result<()> {
    let mut interfaces = crate::net::iface::get_all_interfaces();
    if let Some(iface_name) = &args.iface {
        interfaces.retain(|i| &i.name == iface_name);
    }
    if let Some(ShowAction::Show { target }) = &args.action {
        interfaces.retain(|i| &i.name == target);
    }

    let mut entries: Vec<IpAddrEntry> = interfaces
        .iter()
        .flat_map(super::common::interface_to_ip_entries)
        .collect();

    if args.ipv4 {
        entries.retain(|e| e.family == "ipv4");
    }
    if args.ipv6 {
        entries.retain(|e| e.family == "ipv6");
    }

    entries.sort_by(|a, b| {
        a.iface
            .cmp(&b.iface)
            .then(a.family.cmp(&b.family))
            .then(a.address.cmp(&b.address))
    });

    crate::renderer::render_ip_entries(&entries, &args.out)
}
