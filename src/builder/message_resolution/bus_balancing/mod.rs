use std::cmp::Ordering;

use crate::{builder::bus::BusBuilder, config::TypeRef};

use self::node_receive_set::NodeReceiveSet;

use super::receive_set::{NetworkInfo, ReceiverSetRef};

pub mod node_receive_set;

pub struct BusInfo {
    receive_sets: Vec<ReceiverSetRef>,
    node_sets: Vec<NodeReceiveSet>,
    bus_name: String,
}

impl BusInfo {
    pub fn receive_sets(&self) -> &Vec<ReceiverSetRef> {
        &self.receive_sets
    }
    pub fn node_sets(&self) -> &Vec<NodeReceiveSet> {
        &self.node_sets
    }
    pub fn bus_name(&self) -> &str {
        &self.bus_name
    }
}

pub fn balance_buses(
    network_info: NetworkInfo,
    types: &Vec<TypeRef>,
    buses: &Vec<BusBuilder>,
) -> Vec<BusInfo> {
    let mut bus_receiver_sets: Vec<Vec<ReceiverSetRef>> = vec![];
    for i in 0..buses.len() {
        assert_eq!(i, buses[i].0.borrow().id as usize);
        bus_receiver_sets.push(vec![]);
    }
    let mut any_bus_receiver_sets = vec![];
    for receiver_set in network_info.receive_sets() {
        match receiver_set.identifier().bus() {
            Some(bus_id) => {
                bus_receiver_sets[*bus_id as usize].push(receiver_set.clone());
            }
            None => {
                any_bus_receiver_sets.push(receiver_set.clone());
            }
        }
    }
    let mut any_bus_receiver_sets: Vec<(ReceiverSetRef, f64)> = any_bus_receiver_sets
        .into_iter()
        .map(|rx_set| (rx_set.clone(), rx_set.bus_load(types)))
        .collect();
    // sort by bus load
    any_bus_receiver_sets.sort_by(|&(_, a), &(_, b)| match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        (false, false) => a.partial_cmp(&b).unwrap(),
    });
    // desc -> aesc
    any_bus_receiver_sets.reverse();
    let mut bus_receiver_sets: Vec<(Vec<ReceiverSetRef>, f64)> = bus_receiver_sets
        .into_iter()
        .map(|bus_sets| -> (Vec<ReceiverSetRef>, f64) {
            (
                bus_sets.clone(),
                bus_sets.iter().map(|rx_set| rx_set.bus_load(types)).sum(),
            )
        })
        .collect();

    for any_bus_receiver_set in any_bus_receiver_sets {
        let min = bus_receiver_sets
            .iter_mut()
            .min_by_key(|(_, load)| *load as u64)
            .expect("expected at least one bus_receiver set");
        min.0.push(any_bus_receiver_set.0);
        min.1 += any_bus_receiver_set.1;
    }
    let bus_receiver_sets: Vec<Vec<ReceiverSetRef>> =
        bus_receiver_sets.into_iter().map(|(set, _)| set).collect();

    bus_receiver_sets
        .into_iter()
        .enumerate()
        .map(|(bus_id, set)| BusInfo {
            bus_name: buses[bus_id].0.borrow().name.clone(),
            node_sets: network_info
                .nodes()
                .iter()
                .map(|node| {
                    let node_name = node.0.borrow().name.clone();
                    let rx_sets: Vec<ReceiverSetRef> = set
                        .iter()
                        .map(|rx_set| rx_set.clone())
                        .filter(|rx_set| {
                            rx_set
                                .identifier()
                                .receivers()
                                .iter()
                                .any(|rx| rx.0.borrow().name == node_name)
                        })
                        .collect();
                    NodeReceiveSet::new(node_name, rx_sets)
                })
                .collect(),
            receive_sets: set,
        })
        .collect()
}
