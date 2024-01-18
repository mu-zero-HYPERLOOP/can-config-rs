use std::rc::Rc;

use crate::builder::{
    message_resolution::set_minimization::{
        bucket_layout::BucketLayout, node_receive_set::NodeReceiveSet,
    },
    MessageBuilder, MessagePriority, NodeBuilder,
};

use self::{receiver_set::ReceiverSet, set_identifier::SetIdentifier};

mod bucket_layout;
mod node_receive_set;
mod priority_bucket;
mod receiver_set;
mod set_identifier;
use rand::thread_rng;
use rand::Rng;

const MAX_FILTERS_PER_NODE: usize = 4;
const STD_ID_LENGTH: u32 = 11;

pub struct MinimizedSet {
    messages: [Vec<MessageBuilder>; MessagePriority::count()],
}

impl MinimizedSet {
    pub fn new(messages: [Vec<MessageBuilder>; MessagePriority::count()]) -> Self {
        Self { messages }
    }
}

/**
 * messages is not allowed to contain messages with fixed id assignments!
 */
pub fn calculate_min_sets(messages: &Vec<MessageBuilder>) -> Vec<MinimizedSet> {
    // TODO How correctly handle ext and std frames
    // maybe it would be optimal to minimize them completly seperate!
    let mut receiver_sets: Vec<ReceiverSet> = vec![];

    let mut fixed_messages: Vec<MessageBuilder> = vec![];

    let mut rx_nodes: Vec<NodeBuilder> = vec![];

    for message in messages {
        let bus = message.0.borrow().bus.clone().map(|bus| bus.0.borrow().id);
        let ide = match message.0.borrow().id {
            crate::builder::message_builder::MessageIdTemplate::StdId(_)
            | crate::builder::message_builder::MessageIdTemplate::ExtId(_) => {
                fixed_messages.push(message.clone());
                continue;
            }
            crate::builder::message_builder::MessageIdTemplate::AnyStd(_) => Some(false),
            crate::builder::message_builder::MessageIdTemplate::AnyExt(_) => Some(true),
            crate::builder::message_builder::MessageIdTemplate::AnyAny(_) => None,
        };
        let set_identifier = SetIdentifier::new(&message.0.borrow().receivers, bus, ide);
        for rx in &message.0.borrow().receivers {
            let rx_name: String = rx.0.borrow().name.clone();
            if !rx_nodes.iter().any(|node| node.0.borrow().name == rx_name) {
                rx_nodes.push(rx.clone());
            }
        }
        let set_position = receiver_sets
            .iter()
            .position(|rx_set| rx_set.identifier().eq(&set_identifier));
        match set_position {
            Some(set_position) => receiver_sets[set_position].insert_message(message),
            None => {
                let mut new_receiver_set = ReceiverSet::new(set_identifier);
                new_receiver_set.insert_message(message);
                receiver_sets.push(new_receiver_set);
            }
        }
    }

    let receiver_sets: Vec<Rc<ReceiverSet>> = receiver_sets
        .into_iter()
        .map(|rx_set| Rc::new(rx_set))
        .collect();

    let node_receiver_sets: Vec<NodeReceiveSet> = rx_nodes
        .iter()
        .map(|node| {
            let node_name = node.0.borrow().name.clone();
            let rx_sets: Vec<Rc<ReceiverSet>> = receiver_sets
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
        .collect();
    if node_receiver_sets.is_empty() {
        panic!("What please at leat supply one receiver to a message")
    }
    println!("receive set count: {}", receiver_sets.len());

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

        let reducable_node = node_receiver_sets
            .iter()
            .max_by_key(|node_rx_set| node_rx_set.receive_set_count(&bucket_layout))
            .expect("It was asserted that there exist at least one node receiver set");

        let set_count: usize = receiver_sets
            .iter()
            .map(|rx_set| rx_set.set_count(&bucket_layout))
            .sum();

        for rx_set in &receiver_sets {
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
            None => break,
        }
        // TODO the brake condition is not optimal because it will just reduce
        // sets as much as possible. without considering setcode len!
    }

    let total_set_count: usize = receiver_sets
        .iter()
        .map(|rx_set| rx_set.set_count(&bucket_layout))
        .sum();
    let setcode_len = (total_set_count as f64).log2().ceil() as u32;
    println!("");
    println!("Total setcount : {total_set_count}");
    println!("setcode-len    : {setcode_len}");
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

    let reducable_node = node_receiver_sets
        .iter()
        .max_by_key(|node_rx_set| node_rx_set.receive_set_count(&bucket_layout))
        .expect("It was asserted that there exist at least one node receiver set");

    let max_filters = reducable_node.receive_set_count(&bucket_layout);
    println!("Max filters: {max_filters}");
    println!("");

    // determine receiver set for each fixed message if the set doesn't exist add a new set!

    // apply fixed ids!
    // determine set of fixed messages seperated based on:
    //  - receivers
    //  - setcode suffix (based on current setcode len).
    // Try to insert each fixed set into a existing receiver set! into the associated receiver set!

    let minimized_sets: Vec<MinimizedSet> = receiver_sets
        .iter()
        .map(|rx_set| rx_set.to_sets(&bucket_layout))
        .flatten()
        .collect();

    minimized_sets
}

#[cfg(test)]
mod tests {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::builder::{MessagePriority, NetworkBuilder};

    use super::calculate_min_sets;

    #[test]
    pub fn test_1() {
        let network_builder = NetworkBuilder::new();

        struct MessageNameGen {
            next: String,
        }
        impl MessageNameGen {
            pub fn new() -> Self {
                Self {
                    next: "0".to_owned(),
                }
            }
            pub fn next(&mut self) -> String {
                let next = self.next.clone();
                self.next += "0";
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
        let message_per_node = 250;
        for _ in 0..message_per_node {
            create_test_message(&network_builder, "secu", &mut name_gen);
        }
        for _ in 0..message_per_node {
            create_test_message(&network_builder, "master", &mut name_gen);
        }
        for _ in 0..message_per_node {
            create_test_message(&network_builder, "becu", &mut name_gen);
        }

        let fixed = network_builder.create_message("secu", None);
        fixed.set_std_id(0xFF);

        calculate_min_sets(&network_builder.0.borrow().messages.borrow().clone());

        assert!(false);
    }
}
