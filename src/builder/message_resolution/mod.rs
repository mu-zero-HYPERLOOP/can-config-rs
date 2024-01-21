use crate::{errors, config::TypeRef};

use self::filter_configuration::NodeFilterBank;

use super::{MessageBuilder, bus::BusBuilder, NodeBuilder};


mod set_minimization;
mod set_assignment;
mod receive_set;
mod bus_balancing;
mod setcode_optimization;
mod assign_messages;
mod filter_configuration;
mod logging;


pub struct BusFilterBank {
    node_filters : Vec<NodeFilterBank>,
}

impl BusFilterBank {
    pub fn node_filters(&self) -> &Vec<NodeFilterBank> {
        &self.node_filters
    }
    pub fn node_filter_of(&self, node_name : &str) -> Option<&NodeFilterBank>{
        self.node_filters.iter().find(|nf| &nf.node().0.borrow().name == node_name)
    }

    pub fn node_filter_of_builder(&self, node_builder : &NodeBuilder) -> Option<&NodeFilterBank>{
        let node_name = node_builder.0.borrow().name.clone();
        self.node_filter_of(&node_name)
    }
}

pub fn resolve_ids_filters_and_buses(
    buses: &Vec<BusBuilder>,
    messages: &Vec<MessageBuilder>,
    types: &Vec<TypeRef>,
) -> errors::Result<Vec<BusFilterBank>> {
    let log_info = logging::cache_logging_info(types ,messages);
    let mut bus_filter_banks =  vec![];
    let network_info = receive_set::generate_receive_sets_from_messages(messages);
    let bus_infos = bus_balancing::balance_buses(network_info, types, buses);
    for bus_info in bus_infos {
        let minimized_bus = set_minimization::minimize_sets(bus_info);
        let optimized_bus = setcode_optimization::optimize_sets(minimized_bus);
        let assigned_bus = set_assignment::assign_setcodes(optimized_bus);
        assign_messages::assign_messages(&assigned_bus);
        let filters = filter_configuration::find_filter_configuration(&assigned_bus);
        bus_filter_banks.push(BusFilterBank { node_filters: filters });
    }
    logging::log_info(log_info);
    Ok(bus_filter_banks)
}


#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::builder::{MessagePriority, NetworkBuilder};

    #[test]
    pub fn test_1() {
        let network_builder = NetworkBuilder::new();
        network_builder.create_bus("can0", Some(1000000));
        network_builder.create_bus("can1", Some(1000000));

        struct MessageNameGen {
            acc : u32,
        }
        impl MessageNameGen {
            pub fn new() -> Self {
                Self {
                    acc : 0,
                }
            }
            pub fn next(&mut self) -> String {
                let next = format!("test_msg_{}", self.acc);
                self.acc += 1;
                next
            }
        }

        let mut name_gen = MessageNameGen::new();

        fn create_test_message(
            network_builder: &NetworkBuilder,
            rx_node_name: &str,
            name_gen: &mut MessageNameGen,
        ) {
            let name = name_gen.next();
            let mut hasher = DefaultHasher::new();
            name.hash(&mut hasher);
            let x = network_builder.create_message(&name, None);
            let priority = MessagePriority::from_u32(
                (hasher.finish() % MessagePriority::count() as u64) as u32,
            );
            x.set_any_std_id(priority);
            x.add_receiver(rx_node_name);
        }
        let message_per_node = 100;
        for _ in 0..message_per_node {
            create_test_message(&network_builder, "secu", &mut name_gen);
        }
        for _ in 0..message_per_node {
            create_test_message(&network_builder, "master", &mut name_gen);
        }
        for _ in 0..message_per_node {
            create_test_message(&network_builder, "becu", &mut name_gen);
        }

        for _ in 0..message_per_node {
            create_test_message(&network_builder, "clu", &mut name_gen);
        }

        let fixed = network_builder.create_message("fixed_secu1", None);
        fixed.set_std_id(0xFF);
        //
        let fixed = network_builder.create_message("fixed_master", None);
        fixed.set_std_id(0xFA);
        //
        // let fixed = network_builder.create_message("fixed_clu", None);
        // fixed.set_std_id(0xFB);
        
        // let fixed = network_builder.create_message("fixed_secu2", None);
        // fixed.set_std_id(0xFE);
        //
        // let fixed = network_builder.create_message("fixed_secu3", None);
        // fixed.set_ext_id(0xFD);

        network_builder.build().unwrap();

        assert!(false);

    }
}
