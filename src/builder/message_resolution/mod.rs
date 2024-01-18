use crate::{errors, config::TypeRef};

use super::{MessageBuilder, bus::BusBuilder};


mod set_minimization;


pub fn resolve_ids_filters_and_buses(
    buses: &Vec<BusBuilder>,
    messages: &Vec<MessageBuilder>,
    types: &Vec<TypeRef>,
) -> errors::Result<()> {
    Ok(())
}

