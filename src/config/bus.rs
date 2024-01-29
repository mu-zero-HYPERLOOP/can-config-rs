use super::ConfigRef;



pub type BusRef = ConfigRef<Bus>;

#[derive(Debug)]
pub struct Bus {
    id : u32,
    baudrate : u32,
    name : String,
}

impl Bus {
    pub fn new(name : &str, id : u32, baudrate : u32) -> Self{
        Self {
            id,
            baudrate,
            name : name.to_owned(),
        }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn baudrate(&self) -> u32 {
        self.baudrate
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}

