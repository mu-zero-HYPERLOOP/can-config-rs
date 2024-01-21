use crate::builder::NodeBuilder;

use super::set_assignment::AssignedBus;


pub struct NodeFilterBank {
    filters : Vec<Filter>,
    node : NodeBuilder,
}

impl NodeFilterBank{
    pub fn node(&self) -> &NodeBuilder {
        &self.node
    }
    pub fn filters(&self) -> &Vec<Filter> {
        &self.filters
    }
}

pub struct Filter {
    mask : u32,
    id : u32,
}
impl Filter {
    pub fn mask(&self) -> u32 {
        self.mask
    }
    pub fn id(&self) -> u32 {
        self.id
    }
}


pub fn find_filter_configuration(bus : &AssignedBus) -> Vec<NodeFilterBank> {
    vec![]
}
