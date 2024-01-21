use crate::builder::{MessageBuilder, MessagePriority, NodeBuilder};

use super::set_minimization::{
    bucket_layout::BucketLayout, MinimizedBus,
};

pub struct AssignedBus {
    sets: Vec<AssignedSet>,
    bucket_layout: BucketLayout,
    bus_name : String,
}

impl AssignedBus{
    pub fn sets(&self) -> &Vec<AssignedSet> {
        &self.sets
    }
    pub fn bucket_layout(&self) -> &BucketLayout {
        &self.bucket_layout
    }
    pub fn bus_name(&self) -> &str {
        &self.bus_name
    }
}

pub struct AssignedSet {
    messages: [Vec<MessageBuilder>; MessagePriority::count()],
    setcode : u32,
    setcode_len : u32,
    receivers: Vec<NodeBuilder>,
    ide: bool,
}

impl AssignedSet {
    pub fn messages_with_priority(&self, prio : usize) -> &Vec<MessageBuilder> {
        &self.messages[prio]
    }
    pub fn setcode(&self) -> u32 {
        self.setcode
    }
    pub fn setcode_len(&self) -> u32 {
        self.setcode_len
    }
    pub fn receivers(&self) -> &Vec<NodeBuilder> {
        &self.receivers
    }
    pub fn ide(&self) -> bool {
        self.ide
    }
}

pub fn assign_setcodes(bus_set: MinimizedBus) -> AssignedBus {
    let set_count = bus_set.sets().len();
    println!("setcount : {set_count}");
    let setcode_len = (set_count as f64).log2().ceil() as u32;
    let mut avaiable_setcodes = vec![0;(2usize).pow(setcode_len)];
    for (i, setcode) in avaiable_setcodes.iter_mut().enumerate(){
        *setcode = i as u32;
    }

    let mut assigned_sets : Vec<AssignedSet> = vec![];

    let bus_name = bus_set.bus_name();
    // assign fixed sets!
    for set in bus_set.sets() {
        let ide = false;
        let Some(id_prefix) = set.id().id() else {
            continue
        };
        let setcode = id_prefix & 0xFFFFFFFFu32.overflowing_shr(32 - setcode_len).0;
        println!("fixed setcode = {setcode} {setcode_len}");
        let avai_pos = avaiable_setcodes.iter().position(|&s| s == setcode).expect("setcode prefix of fixed id is not available");
        avaiable_setcodes.remove(avai_pos);

        let receivers = set.id().receivers().clone();
        assigned_sets.push(AssignedSet { setcode, setcode_len, receivers, ide, messages: set.messages().clone() })
    }

    for set in bus_set.sets() {
        let ide = false;
        let None = set.id().id() else {
            continue
        };
        let setcode = *avaiable_setcodes.last().expect("not enought setcodes avaiable");
        avaiable_setcodes.pop();

        let receivers = set.id().receivers().clone();
        assigned_sets.push(AssignedSet { setcode, setcode_len, receivers, ide, messages: set.messages().clone() })
    }
    AssignedBus { bus_name : bus_name.to_owned(), sets: assigned_sets, bucket_layout: bus_set.into_bucket_layout()}
}
