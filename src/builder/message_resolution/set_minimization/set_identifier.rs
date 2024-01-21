use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};

use crate::builder::NodeBuilder;
use std::fmt::Debug;

#[derive(Clone)]
pub struct SetIdentifier {
    receivers: Vec<NodeBuilder>,
    bus: Option<u32>,
    ide: Option<bool>,
    id : Option<u32>,
    hashcode: u64,
}

impl SetIdentifier {
    pub fn new(
        receivers: &Vec<NodeBuilder>,
        bus: Option<u32>,
        ide: Option<bool>,
        id : Option<u32>,
    ) -> Self {
        let mut receivers = receivers.clone();
        receivers.sort_by_key(|r| r.0.borrow().name.clone());
        let mut hasher = DefaultHasher::new();
        for rx in &receivers {
            rx.0.borrow().name.hash(&mut hasher);
        }
        bus.hash(&mut hasher);
        ide.hash(&mut hasher);
        id.hash(&mut hasher);
        Self {
            receivers,
            bus,
            ide,
            hashcode : hasher.finish(),
            id,
        }
    }
    pub fn bus(&self) -> &Option<u32> {
        &self.bus
    }
    pub fn ide(&self) -> &Option<bool> {
        &self.ide
    }
    pub fn id(&self) -> &Option<u32> {
        &self.id
    }
    pub fn receivers(&self) -> &Vec<NodeBuilder> {
        &self.receivers
    }
    pub fn compressable(&self) -> bool {
        self.id.is_none()
    }
}

impl Hash for SetIdentifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hashcode)
    }
}

impl PartialEq for SetIdentifier {
    fn eq(&self, other: &Self) -> bool {
        if other.hashcode != self.hashcode {
            return false;
        }
        if other.bus != self.bus {
            return false;
        }
        if other.ide != self.ide {
            return false;
        }
        if other.id != self.id {
            return false;
        }
        for (a,b) in std::iter::zip(other.receivers.iter(), self.receivers.iter()) {
            let name_a = a.0.borrow().name.clone();
            if name_a != b.0.borrow().name {
                return false;
            }
        }
        return true;
    }
}

impl Debug for SetIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for rx in self.receivers() {
            let rx_name = &rx.0.borrow().name;
            write!(f, "{rx_name},")?;
        }
        write!(f, "]")?;
        match self.ide {
            Some(ide) => {
                if ide {
                    write!(f, "x")?;
                }
            }
            None => {
                write!(f, "?")?;
            }
        }
        Ok(())
    }
}

impl Eq for SetIdentifier { }
