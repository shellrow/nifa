use anyhow::Result;
use netdev::interface::state::OperState;

use crate::cli::LinkArgs;
use crate::model::IfDetail;

pub fn run(args: &LinkArgs) -> Result<()> {
    let mut interfaces = crate::net::iface::get_all_interfaces();

    if let Some(iface_name) = &args.iface {
        interfaces.retain(|iface| &iface.name == iface_name);
    }
    if args.up {
        interfaces.retain(|iface| iface.oper_state == OperState::Up);
    }
    if args.down {
        interfaces.retain(|iface| iface.oper_state == OperState::Down);
    }

    interfaces.sort_by(|a, b| a.name.cmp(&b.name));
    let details: Vec<IfDetail> = interfaces
        .iter()
        .map(|iface| super::common::interface_to_detail(iface, false))
        .collect();

    crate::renderer::render_link_entries(&details, &args.out)
}
