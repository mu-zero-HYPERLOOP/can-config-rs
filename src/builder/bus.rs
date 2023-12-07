use super::{BuilderRef, make_builder_ref};


#[derive(Debug, Clone)]
pub struct BusBuilder(pub BuilderRef<BusData>);

#[derive(Debug)]
pub struct BusData {
    pub name : String,
    pub id : u32,
    pub baudrate : u32,
    pub expected_utilization : u32,
}

impl BusBuilder {
    pub fn new(name : &str, id : u32) -> Self {
        BusBuilder(make_builder_ref(BusData {
            name : name.to_owned(),
            id,
            baudrate : 1000000,
            expected_utilization : 0,
        }))
    }

    pub fn baudrate(&self, baudrate : u32) {
        self.0.borrow_mut().baudrate = baudrate;
    }
}

