use super::ConfigRef;



pub type BusRef = ConfigRef<Bus>;

#[derive(Debug)]
pub struct Bus {
    id : u32,
    baudrate : u32,
}

impl Bus {
    pub fn new(id : u32, baudrate : u32) -> Self{
        Self {
            id,
            baudrate
        }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn baudrate(&self) -> u32 {
        self.baudrate
    }
}

