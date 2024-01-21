use self::set_identifier::SetIdentifier;
use crate::builder::{
        message_resolution::{set_minimization::bucket_layout::BucketLayout, bus_balancing::node_receive_set::NodeReceiveSet},
        MessageBuilder,
    };
use crate::builder::MessagePriority;

use super::bus_balancing::BusInfo;

pub mod bucket_layout;
pub mod priority_bucket;
pub mod set_identifier;

const MAX_FILTERS_PER_NODE: usize = 8;
const STD_ID_LENGTH: u32 = 11;

pub struct MinimizedSet {
    messages: [Vec<MessageBuilder>; MessagePriority::count()],
    id: SetIdentifier,
}

impl MinimizedSet {
    pub fn new(
        messages: [Vec<MessageBuilder>; MessagePriority::count()],
        id: SetIdentifier,
    ) -> Self {
        Self { messages, id }
    }
    pub fn messages_with_priority(&self, priority: usize) -> &Vec<MessageBuilder> {
        &self.messages[priority]
    }
    pub fn messages(&self) -> &[Vec<MessageBuilder>; MessagePriority::count()] {
        &self.messages
    }
    pub fn id(&self) -> &SetIdentifier {
        &self.id
    }
}

pub struct MinimizedBus {
    bus_name: String,
    sets: Vec<MinimizedSet>,
    bucket_layout: BucketLayout,
}

impl MinimizedBus {
    pub fn new(bus_name: String, sets: Vec<MinimizedSet>, bucket_layout: BucketLayout) -> Self {
        Self {
            bus_name,
            sets,
            bucket_layout,
        }
    }
    pub fn bucket_layout(&self) -> &BucketLayout {
        &self.bucket_layout
    }
    pub fn into_bucket_layout(self) -> BucketLayout {
        self.bucket_layout
    }
    pub fn sets(&self) -> &Vec<MinimizedSet> {
        &self.sets
    }
    pub fn bus_name(&self) -> &str {
        &self.bus_name
    }
}

/**
 * messages is not allowed to contain messages with fixed id assignments!
 */

pub fn minimize_sets(bus_info: BusInfo) -> MinimizedBus {
    if bus_info.node_sets().is_empty() {
        panic!("Can't minimize the sets for a bus if all messages on the bus are not received at all!");
    }
    println!("receive set count: {}", bus_info.receive_sets().len());

    let mut bucket_layout = BucketLayout::new();

    let mut it = 0;
    loop {
        println!("\nBegin Iteration {it}");
        it += 1;

        println!("Bucket Stats:");
        println!("-realtime    : {}", bucket_layout.bucket_size(0));
        println!("-high        : {}", bucket_layout.bucket_size(1));
        println!("-normal      : {}", bucket_layout.bucket_size(2));
        println!("-low         : {}", bucket_layout.bucket_size(3));
        println!("-super-low   : {}", bucket_layout.bucket_size(4));

        let reducable_node = bus_info.node_sets()
            .iter()
            .max_by_key(|node_rx_set| node_rx_set.receive_set_count(&bucket_layout))
            .expect("It was asserted that there exist at least one node receiver set");

        let set_count: usize = bus_info.receive_sets()
            .iter()
            .map(|rx_set| rx_set.set_count(&bucket_layout))
            .sum();

        for rx_set in bus_info.receive_sets() {
            println!("RxSet {:?}:", rx_set.identifier());
            println!("-set-count : {}", rx_set.set_count(&bucket_layout));
            println!(
                "-realtime  : {} -> {}",
                rx_set.priorioty_bucket(0).message_count(),
                rx_set
                    .priorioty_bucket(0)
                    .required_sets(bucket_layout.bucket_size(0))
            );
            println!(
                "--required-inc = {}",
                rx_set
                    .priorioty_bucket(0)
                    .required_inc_for_merge(bucket_layout.bucket_size(0))
                    .unwrap_or_default()
            );
            println!(
                "-high      : {} -> {}",
                rx_set.priorioty_bucket(1).message_count(),
                rx_set
                    .priorioty_bucket(1)
                    .required_sets(bucket_layout.bucket_size(1))
            );
            println!(
                "--required-inc = {}",
                rx_set
                    .priorioty_bucket(1)
                    .required_inc_for_merge(bucket_layout.bucket_size(1))
                    .unwrap_or_default()
            );
            println!(
                "-normal    : {} -> {}",
                rx_set.priorioty_bucket(2).message_count(),
                rx_set
                    .priorioty_bucket(2)
                    .required_sets(bucket_layout.bucket_size(2))
            );
            println!(
                "--required-inc = {}",
                rx_set
                    .priorioty_bucket(2)
                    .required_inc_for_merge(bucket_layout.bucket_size(2))
                    .unwrap_or_default()
            );
            println!(
                "-low       : {} -> {}",
                rx_set.priorioty_bucket(3).message_count(),
                rx_set
                    .priorioty_bucket(3)
                    .required_sets(bucket_layout.bucket_size(3))
            );
            println!(
                "--required-inc = {}",
                rx_set
                    .priorioty_bucket(3)
                    .required_inc_for_merge(bucket_layout.bucket_size(3))
                    .unwrap_or_default()
            );
            println!(
                "-superlow  : {} -> {}",
                rx_set.priorioty_bucket(4).message_count(),
                rx_set
                    .priorioty_bucket(4)
                    .required_sets(bucket_layout.bucket_size(4))
            );
            println!(
                "--required-inc = {}",
                rx_set
                    .priorioty_bucket(4)
                    .required_inc_for_merge(bucket_layout.bucket_size(4))
                    .unwrap_or_default()
            );
        }

        assert!(set_count != 0, "required for usize::ilog2");
        let setcode_len = (set_count as f64).log2().ceil() as u32;
        let prio_len = bucket_layout.prio_bit_size();
        let id_len = setcode_len + prio_len;
        let max_filters = reducable_node.receive_set_count(&bucket_layout);

        println!("Result:");
        println!("-set-count   : {set_count}");
        println!("-setcode_len : {setcode_len}");
        println!("-prio_len    : {prio_len}");
        println!("-unused-bits : {}", 11 as i32 - id_len as i32);
        println!("-max_filters : {max_filters}");

        if id_len <= STD_ID_LENGTH && max_filters <= MAX_FILTERS_PER_NODE {
            break;
        }

        let best_commit = reducable_node
            .receive_sets()
            .iter()
            .map(|rx_set| rx_set.min_commit_to_merge(&bucket_layout))
            .flatten()
            .min_by_key(|commit| commit.count());
        match best_commit {
            Some(best_commit) => {
                println!("APPLY_COMMIT:");
                println!("-realtime-inc : {}", best_commit.inc()[0]);
                println!("-high-inc     : {}", best_commit.inc()[1]);
                println!("-normal-inc   : {}", best_commit.inc()[2]);
                println!("-low-inc      : {}", best_commit.inc()[3]);
                println!("-superlow-inc : {}", best_commit.inc()[4]);
                bucket_layout.apply_commit(best_commit)
            }
            None => {
                println!("WARNING : exit without finding valid id assignment");
                break;
            }
        }
    }

    let total_set_count: usize = bus_info.receive_sets()
        .iter()
        .map(|rx_set| rx_set.set_count(&bucket_layout))
        .sum();
    let setcode_len = (total_set_count as f64).log2().ceil() as u32;
    println!("");
    println!("Bucket Stats:");
    println!("-realtime    : {}", bucket_layout.bucket_size(0));
    println!("-high        : {}", bucket_layout.bucket_size(1));
    println!("-normal      : {}", bucket_layout.bucket_size(2));
    println!("-low         : {}", bucket_layout.bucket_size(3));
    println!("-super-low   : {}", bucket_layout.bucket_size(4));

    let total_priority_count = bucket_layout.total_bucket_size();
    println!("Combined bucket count : {total_priority_count}");
    let total_priority_bits = (total_priority_count as f64).log2().ceil() as u32;
    println!("Priority bit count    : {total_priority_bits}");

    let reducable_node = bus_info.node_sets()
        .iter()
        .max_by_key(|node_rx_set| node_rx_set.receive_set_count(&bucket_layout))
        .expect("It was asserted that there exist at least one node receiver set");

    let max_filters = reducable_node.receive_set_count(&bucket_layout);
    println!("Total setcount : {total_set_count}");
    println!("setcode-len    : {setcode_len}");
    println!("Max filters: {max_filters}");
    println!("");

    let minimized_sets: Vec<MinimizedSet> = bus_info.receive_sets()
        .iter()
        .map(|rx_set| rx_set.to_minimized_sets(&bucket_layout))
        .flatten()
        .collect();

    MinimizedBus::new(bus_info.bus_name().to_owned(), minimized_sets, bucket_layout)
}

// pub fn calculate_min_sets(
//     buses: &Vec<BusBuilder>,
//     messages: &Vec<MessageBuilder>,
//     types: &Vec<TypeRef>,
// ) -> Vec<MinimizedBus> {
//     let mut receiver_sets: Vec<ReceiverSet> = vec![];
//     let mut rx_nodes: Vec<NodeBuilder> = vec![];
//     for message in messages {
//         let bus = message.0.borrow().bus.clone().map(|bus| bus.0.borrow().id);
//         let (ide, id) = match message.0.borrow().id {
//             crate::builder::message_builder::MessageIdTemplate::StdId(id) => {
//                 (Some(false), Some(id))
//             }
//             crate::builder::message_builder::MessageIdTemplate::ExtId(id) => (Some(true), Some(id)),
//             crate::builder::message_builder::MessageIdTemplate::AnyStd(_) => (Some(false), None),
//             crate::builder::message_builder::MessageIdTemplate::AnyExt(_) => (Some(true), None),
//             crate::builder::message_builder::MessageIdTemplate::AnyAny(_) => (None, None),
//         };
//         let set_identifier = SetIdentifier::new(&message.0.borrow().receivers, bus, ide, id);
//         for rx in &message.0.borrow().receivers {
//             let rx_name: String = rx.0.borrow().name.clone();
//             if !rx_nodes.iter().any(|node| node.0.borrow().name == rx_name) {
//                 rx_nodes.push(rx.clone());
//             }
//         }
//         let set_position = receiver_sets
//             .iter()
//             .position(|rx_set| rx_set.identifier().eq(&set_identifier));
//         match set_position {
//             Some(set_position) => receiver_sets[set_position].insert_message(message),
//             None => {
//                 let mut new_receiver_set = ReceiverSet::new(set_identifier);
//                 new_receiver_set.insert_message(message);
//                 receiver_sets.push(new_receiver_set);
//             }
//         }
//     }
//
//     let receiver_sets: Vec<Rc<ReceiverSet>> = receiver_sets
//         .into_iter()
//         .map(|rx_set| Rc::new(rx_set))
//         .collect();
//
//     // assign receiver_sets to buses!
//     let mut bus_receiver_sets: Vec<Vec<Rc<ReceiverSet>>> = vec![];
//     for i in 0..buses.len() {
//         assert_eq!(i, buses[i].0.borrow().id as usize);
//         bus_receiver_sets.push(vec![]);
//     }
//     let mut any_bus_receiver_sets = vec![];
//     for receiver_set in &receiver_sets {
//         match receiver_set.identifier().bus() {
//             Some(bus_id) => {
//                 bus_receiver_sets[*bus_id as usize].push(receiver_set.clone());
//             }
//             None => {
//                 any_bus_receiver_sets.push(receiver_set.clone());
//             }
//         }
//     }
//     let mut any_bus_receiver_sets: Vec<(Rc<ReceiverSet>, f64)> = any_bus_receiver_sets
//         .into_iter()
//         .map(|rx_set| (rx_set.clone(), rx_set.bus_load(types)))
//         .collect();
//     // sort by bus load
//     any_bus_receiver_sets.sort_by(|&(_, a), &(_, b)| match (a.is_nan(), b.is_nan()) {
//         (true, true) => Ordering::Equal,
//         (true, false) => Ordering::Greater,
//         (false, true) => Ordering::Less,
//         (false, false) => a.partial_cmp(&b).unwrap(),
//     });
//     // desc -> aesc
//     any_bus_receiver_sets.reverse();
//     let mut bus_receiver_sets: Vec<(Vec<Rc<ReceiverSet>>, f64)> = bus_receiver_sets
//         .into_iter()
//         .map(|bus_sets| -> (Vec<Rc<ReceiverSet>>, f64) {
//             (
//                 bus_sets.clone(),
//                 bus_sets.iter().map(|rx_set| rx_set.bus_load(types)).sum(),
//             )
//         })
//         .collect();
//
//     for any_bus_receiver_set in any_bus_receiver_sets {
//         let min = bus_receiver_sets
//             .iter_mut()
//             .min_by_key(|(_, load)| *load as u64)
//             .expect("expected at least one bus_receiver set");
//         min.0.push(any_bus_receiver_set.0);
//         min.1 += any_bus_receiver_set.1;
//     }
//     let bus_receiver_sets: Vec<Vec<Rc<ReceiverSet>>> =
//         bus_receiver_sets.into_iter().map(|(set, _)| set).collect();
//
//     let mut minimized_bus_sets: Vec<(Vec<MinimizedSet>, BucketLayout)> = vec![];
//
//     for receiver_sets in bus_receiver_sets {
//         let node_receiver_sets: Vec<NodeReceiveSet> = rx_nodes
//             .iter()
//             .map(|node| {
//                 let node_name = node.0.borrow().name.clone();
//                 let rx_sets: Vec<Rc<ReceiverSet>> = receiver_sets
//                     .iter()
//                     .map(|rx_set| rx_set.clone())
//                     .filter(|rx_set| {
//                         rx_set
//                             .identifier()
//                             .receivers()
//                             .iter()
//                             .any(|rx| rx.0.borrow().name == node_name)
//                     })
//                     .collect();
//                 NodeReceiveSet::new(node_name, rx_sets)
//             })
//             .collect();
//         if node_receiver_sets.is_empty() {
//             panic!("What please at leat supply one receiver to a message")
//         }
//         println!("receive set count: {}", receiver_sets.len());
//
//         let mut bucket_layout = BucketLayout::new();
//
//         let mut it = 0;
//         loop {
//             println!("\nBegin Iteration {it}");
//             it += 1;
//
//             println!("Bucket Stats:");
//             println!("-realtime    : {}", bucket_layout.bucket_size(0));
//             println!("-high        : {}", bucket_layout.bucket_size(1));
//             println!("-normal      : {}", bucket_layout.bucket_size(2));
//             println!("-low         : {}", bucket_layout.bucket_size(3));
//             println!("-super-low   : {}", bucket_layout.bucket_size(4));
//
//             let reducable_node = node_receiver_sets
//                 .iter()
//                 .max_by_key(|node_rx_set| node_rx_set.receive_set_count(&bucket_layout))
//                 .expect("It was asserted that there exist at least one node receiver set");
//
//             let set_count: usize = receiver_sets
//                 .iter()
//                 .map(|rx_set| rx_set.set_count(&bucket_layout))
//                 .sum();
//
//             for rx_set in &receiver_sets {
//                 println!("RxSet {:?}:", rx_set.identifier());
//                 println!("-set-count : {}", rx_set.set_count(&bucket_layout));
//                 println!(
//                     "-realtime  : {} -> {}",
//                     rx_set.priorioty_bucket(0).message_count(),
//                     rx_set
//                         .priorioty_bucket(0)
//                         .required_sets(bucket_layout.bucket_size(0))
//                 );
//                 println!(
//                     "--required-inc = {}",
//                     rx_set
//                         .priorioty_bucket(0)
//                         .required_inc_for_merge(bucket_layout.bucket_size(0))
//                         .unwrap_or_default()
//                 );
//                 println!(
//                     "-high      : {} -> {}",
//                     rx_set.priorioty_bucket(1).message_count(),
//                     rx_set
//                         .priorioty_bucket(1)
//                         .required_sets(bucket_layout.bucket_size(1))
//                 );
//                 println!(
//                     "--required-inc = {}",
//                     rx_set
//                         .priorioty_bucket(1)
//                         .required_inc_for_merge(bucket_layout.bucket_size(1))
//                         .unwrap_or_default()
//                 );
//                 println!(
//                     "-normal    : {} -> {}",
//                     rx_set.priorioty_bucket(2).message_count(),
//                     rx_set
//                         .priorioty_bucket(2)
//                         .required_sets(bucket_layout.bucket_size(2))
//                 );
//                 println!(
//                     "--required-inc = {}",
//                     rx_set
//                         .priorioty_bucket(2)
//                         .required_inc_for_merge(bucket_layout.bucket_size(2))
//                         .unwrap_or_default()
//                 );
//                 println!(
//                     "-low       : {} -> {}",
//                     rx_set.priorioty_bucket(3).message_count(),
//                     rx_set
//                         .priorioty_bucket(3)
//                         .required_sets(bucket_layout.bucket_size(3))
//                 );
//                 println!(
//                     "--required-inc = {}",
//                     rx_set
//                         .priorioty_bucket(3)
//                         .required_inc_for_merge(bucket_layout.bucket_size(3))
//                         .unwrap_or_default()
//                 );
//                 println!(
//                     "-superlow  : {} -> {}",
//                     rx_set.priorioty_bucket(4).message_count(),
//                     rx_set
//                         .priorioty_bucket(4)
//                         .required_sets(bucket_layout.bucket_size(4))
//                 );
//                 println!(
//                     "--required-inc = {}",
//                     rx_set
//                         .priorioty_bucket(4)
//                         .required_inc_for_merge(bucket_layout.bucket_size(4))
//                         .unwrap_or_default()
//                 );
//             }
//
//             assert!(set_count != 0, "required for usize::ilog2");
//             let setcode_len = (set_count as f64).log2().ceil() as u32;
//             let prio_len = bucket_layout.prio_bit_size();
//             let id_len = setcode_len + prio_len;
//             let max_filters = reducable_node.receive_set_count(&bucket_layout);
//
//             println!("Result:");
//             println!("-set-count   : {set_count}");
//             println!("-setcode_len : {setcode_len}");
//             println!("-prio_len    : {prio_len}");
//             println!("-unused-bits : {}", 11 as i32 - id_len as i32);
//             println!("-max_filters : {max_filters}");
//
//             if id_len <= STD_ID_LENGTH && max_filters <= MAX_FILTERS_PER_NODE {
//                 break;
//             }
//
//             let best_commit = reducable_node
//                 .receive_sets()
//                 .iter()
//                 .map(|rx_set| rx_set.min_commit_to_merge(&bucket_layout))
//                 .flatten()
//                 .min_by_key(|commit| commit.count());
//             match best_commit {
//                 Some(best_commit) => {
//                     println!("APPLY_COMMIT:");
//                     println!("-realtime-inc : {}", best_commit.inc()[0]);
//                     println!("-high-inc     : {}", best_commit.inc()[1]);
//                     println!("-normal-inc   : {}", best_commit.inc()[2]);
//                     println!("-low-inc      : {}", best_commit.inc()[3]);
//                     println!("-superlow-inc : {}", best_commit.inc()[4]);
//                     bucket_layout.apply_commit(best_commit)
//                 }
//                 None => {
//                     println!("WARNING : exit without finding valid id assignment");
//                     break;
//                 }
//             }
//         }
//
//         let total_set_count: usize = receiver_sets
//             .iter()
//             .map(|rx_set| rx_set.set_count(&bucket_layout))
//             .sum();
//         let setcode_len = (total_set_count as f64).log2().ceil() as u32;
//         println!("");
//         println!("Bucket Stats:");
//         println!("-realtime    : {}", bucket_layout.bucket_size(0));
//         println!("-high        : {}", bucket_layout.bucket_size(1));
//         println!("-normal      : {}", bucket_layout.bucket_size(2));
//         println!("-low         : {}", bucket_layout.bucket_size(3));
//         println!("-super-low   : {}", bucket_layout.bucket_size(4));
//
//         let total_priority_count = bucket_layout.total_bucket_size();
//         println!("Combined bucket count : {total_priority_count}");
//         let total_priority_bits = (total_priority_count as f64).log2().ceil() as u32;
//         println!("Priority bit count    : {total_priority_bits}");
//
//         let reducable_node = node_receiver_sets
//             .iter()
//             .max_by_key(|node_rx_set| node_rx_set.receive_set_count(&bucket_layout))
//             .expect("It was asserted that there exist at least one node receiver set");
//
//         let max_filters = reducable_node.receive_set_count(&bucket_layout);
//         println!("Total setcount : {total_set_count}");
//         println!("setcode-len    : {setcode_len}");
//         println!("Max filters: {max_filters}");
//         println!("");
//
//         let minimized_sets: Vec<MinimizedSet> = receiver_sets
//             .iter()
//             .map(|rx_set| rx_set.to_minimized_sets(&bucket_layout))
//             .flatten()
//             .collect();
//         minimized_bus_sets.push((minimized_sets, bucket_layout));
//     }
//
//     minimized_bus_sets
//         .into_iter()
//         .enumerate()
//         .map(|(bus_id, (sets, bucket_layout))| {
//             MinimizedBus::new(bus_id as u32, sets, bucket_layout)
//         })
//         .collect()
// }

