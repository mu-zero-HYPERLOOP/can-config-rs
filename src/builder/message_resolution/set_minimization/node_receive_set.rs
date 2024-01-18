use std::rc::Rc;

use super::{receiver_set::ReceiverSet, bucket_layout::BucketLayout};



pub struct NodeReceiveSet {
    node_name : String,
    receiver_sets : Vec<Rc<ReceiverSet>>
}

impl NodeReceiveSet {
    pub fn new(node_name : String, receiver_sets : Vec<Rc<ReceiverSet>>) -> Self{
        Self {
            node_name,
            receiver_sets,
        }
    }
    pub fn node_name(&self) -> &str {
        &self.node_name
    }
    pub fn receive_sets(&self) -> &Vec<Rc<ReceiverSet>> {
        &self.receiver_sets
    }
    pub fn receive_set_count(&self, bucket_layout : &BucketLayout) -> usize {
        self.receiver_sets.iter().map(|rx_set| rx_set.set_count(bucket_layout)).sum()
    }
}
