use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    hash::{Hash, Hasher},
};

use crate::errors;

use super::{bus::BusBuilder, MessageBuilder};

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
enum BusAssignment {
    Bus { id: u32 },
    Any,
}

impl BusAssignment {
    pub fn new(message: &MessageBuilder) -> Self {
        let message_data = message.0.borrow();
        match &message_data.bus {
            Some(bus) => BusAssignment::Bus {
                id: bus.0.borrow().id,
            },
            None => BusAssignment::Any,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
enum TypeAssignment {
    Std,
    Ext,
    Any,
}

impl TypeAssignment {
    pub fn new(message: &MessageBuilder) -> Self {
        let message_data = message.0.borrow();
        match message_data.id {
            super::message_builder::MessageIdTemplate::StdId(_) => TypeAssignment::Std,
            super::message_builder::MessageIdTemplate::ExtId(_) => TypeAssignment::Ext,
            super::message_builder::MessageIdTemplate::AnyStd(_) => TypeAssignment::Std,
            super::message_builder::MessageIdTemplate::AnyExt(_) => TypeAssignment::Ext,
            super::message_builder::MessageIdTemplate::AnyAny(_) => TypeAssignment::Any,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
enum SuffixAssignment {
    Suffix { value: u32 },
    None,
}

impl SuffixAssignment {
    pub fn new(message: &MessageBuilder) -> Self {
        let message_data = message.0.borrow();
        match message_data.id {
            super::message_builder::MessageIdTemplate::StdId(id) => {
                SuffixAssignment::Suffix { value: id }
            }
            super::message_builder::MessageIdTemplate::ExtId(id) => {
                SuffixAssignment::Suffix { value: id }
            }
            super::message_builder::MessageIdTemplate::AnyStd(_) => SuffixAssignment::None,
            super::message_builder::MessageIdTemplate::AnyExt(_) => SuffixAssignment::None,
            super::message_builder::MessageIdTemplate::AnyAny(_) => SuffixAssignment::None,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
struct ReceiverSet {
    set: HashSet<String>,
    hash: u64,
}

impl ReceiverSet {
    fn new(message: &MessageBuilder) -> Self {
        let message_data = message.0.borrow();
        let receivers = &message_data.receivers;
        let mut set = HashSet::new();
        let mut hasher = DefaultHasher::new();
        let mut sorted = vec![];
        for rx in receivers {
            let rx_name = rx.0.borrow().name.clone();
            set.insert(rx_name.clone());
            sorted.push(rx_name.clone());
        }
        sorted.sort();
        for rx_name in sorted {
            rx_name.hash(&mut hasher);
        }

        Self {
            set,
            hash: hasher.finish(),
        }
    }
}

impl Hash for ReceiverSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash)
    }
}

struct CombineOptions {
    allow_ext: bool,
    std_suffix_len: u32,
    ext_suffix_len: u32,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
struct SetKey {
    bus_assignment: BusAssignment,
    type_assignment: TypeAssignment,
    suffix_assignment: SuffixAssignment,
    receiver_set: ReceiverSet,
}

impl SetKey {
    fn new(message: &MessageBuilder) -> Self {
        Self {
            bus_assignment: BusAssignment::new(message),
            type_assignment: TypeAssignment::new(message),
            suffix_assignment: SuffixAssignment::new(message),
            receiver_set: ReceiverSet::new(message),
        }
    }

    fn combine(a: &SetKey, b: &SetKey, options: &CombineOptions) -> Option<SetKey> {
        if a.receiver_set != b.receiver_set {
            return None; // 2 sets can't be merged if their receive set is different
        }
        let type_assignment = match a.type_assignment {
            TypeAssignment::Std => {
                match b.type_assignment {
                    TypeAssignment::Std => TypeAssignment::Std,
                    TypeAssignment::Ext => return None, //can't merge sets of differnt types
                    TypeAssignment::Any => TypeAssignment::Std,
                }
            }
            TypeAssignment::Ext => {
                match b.type_assignment {
                    TypeAssignment::Std => return None, //can't merge sets of differnt types
                    TypeAssignment::Ext => TypeAssignment::Ext,
                    TypeAssignment::Any if options.allow_ext => TypeAssignment::Ext,
                    TypeAssignment::Any => return None, // dont allow ext
                }
            }
            TypeAssignment::Any => {
                match b.type_assignment {
                    TypeAssignment::Std => TypeAssignment::Std,
                    TypeAssignment::Ext if options.allow_ext => TypeAssignment::Ext,
                    TypeAssignment::Ext => return None, //dont allow ext
                    TypeAssignment::Any if options.allow_ext => TypeAssignment::Any,
                    TypeAssignment::Any => TypeAssignment::Std, //prefer std if ext is not allowed
                }
            }
        };

        let bus_assignment = match a.bus_assignment {
            BusAssignment::Bus { id: a_bus_id } => {
                match b.bus_assignment {
                    BusAssignment::Bus { id: b_bus_id } if a_bus_id == b_bus_id => {
                        BusAssignment::Bus { id: a_bus_id }
                    }
                    BusAssignment::Bus { id: _ } => return None, // different buses
                    BusAssignment::Any => BusAssignment::Bus { id: a_bus_id },
                }
            }
            BusAssignment::Any => match b.bus_assignment {
                BusAssignment::Bus { id: b_bus_id } => BusAssignment::Bus { id: b_bus_id },
                BusAssignment::Any => BusAssignment::Any,
            },
        };

        let suffix_assignment = match a.suffix_assignment {
            SuffixAssignment::Suffix { value: suffix_a } => {
                match b.suffix_assignment {
                    SuffixAssignment::Suffix { value: suffix_b } => {
                        // compare suffixes for the len
                        let suffix_mask = match type_assignment {
                            TypeAssignment::Std => {
                                (0xFFFFFFFF as u32)
                                    .overflowing_shl(11 - options.std_suffix_len)
                                    .0
                            }
                            TypeAssignment::Ext => {
                                (0xFFFFFFFF as u32)
                                    .overflowing_shl(29 - options.std_suffix_len)
                                    .0
                            }
                            TypeAssignment::Any => {
                                panic!("if a suffix is specified the frame must have a type")
                            }
                        };
                        if (suffix_a & suffix_mask) == (suffix_b & suffix_mask) {
                            SuffixAssignment::Suffix {
                                value: (suffix_a & suffix_mask),
                            }
                        } else {
                            return None;
                        }
                    }
                    SuffixAssignment::None => SuffixAssignment::Suffix { value: suffix_a },
                }
            }
            SuffixAssignment::None => match b.suffix_assignment {
                SuffixAssignment::Suffix { value: suffix_b } => {
                    SuffixAssignment::Suffix { value: suffix_b }
                }
                SuffixAssignment::None => SuffixAssignment::None,
            },
        };

        Some(SetKey {
            receiver_set: a.receiver_set.clone(),
            bus_assignment,
            suffix_assignment,
            type_assignment,
        })
    }
}

struct SetMerge {
    new_key: SetKey,
    assign_bus: Option<u32>,
    assign_std: bool,
    assign_ext: bool,
}

#[derive(Clone)]
struct MessageSet {
    key: SetKey,
    messages: Vec<MessageBuilder>,
}

impl std::fmt::Debug for MessageSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.key)
    }
}

impl MessageSet {
    pub fn new(key: SetKey) -> Self {
        Self {
            key,
            messages: vec![],
        }
    }
    pub fn add_message(&mut self, message: &MessageBuilder) {
        assert_eq!(SetKey::new(message), self.key);
        self.messages.push(message.clone());
    }
}

struct MessageSetSet {
    sets: HashMap<SetKey, MessageSet>,
}
impl std::fmt::Debug for MessageSetSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.sets.values())
    }
}

impl MessageSetSet {
    pub fn new() -> Self {
        Self {
            sets: HashMap::new(),
        }
    }
    pub fn insert(&mut self, message: &MessageBuilder) {
        let key = SetKey::new(message);
        let set = self.sets.get_mut(&key);
        match set {
            Some(set) => {
                set.add_message(message);
            }
            None => {
                let mut set = MessageSet::new(key.clone());
                set.add_message(message);
                self.sets.insert(key, set);
            }
        }
    }

    pub fn merge_sets(&mut self) {
        let sets: Vec<MessageSet> = self.sets.values().map(|s| s.clone()).collect();
        let mut ext_set_count = 0;
        let mut std_set_count = 0;
        let set_count = self.sets.len();
        for (key, set) in &self.sets {
            match key.type_assignment {
                TypeAssignment::Std => std_set_count += 1,
                TypeAssignment::Ext => ext_set_count += 1,
                TypeAssignment::Any => (),
            }
        }
        let ext_suffix_len = (std_set_count as f64).log2().ceil() as u32;
        let std_suffix_len = (ext_set_count as f64).log2().ceil() as u32;

        // suffix collisions!

        let options = CombineOptions {
            allow_ext: false,
            std_suffix_len,
            ext_suffix_len,
        };

        #[derive(Debug)]
        struct Merge {
            key: SetKey,
            i: usize,
            j: usize,
        }

        let mut possible_merges: Vec<Merge> = vec![];

        for i in 0..set_count {
            for j in 0..set_count {
                if i == j {
                    continue;
                }
                let combination = SetKey::combine(&sets[i].key, &sets[j].key, &options);
                match combination {
                    Some(key) => possible_merges.push(Merge { key, i, j }),
                    None => (),
                }
            }
        }

        // evaulate the possible merges based on!
        println!("merges = {possible_merges:?}");
    }
}

pub fn resolve_ids_filters_and_buses(
    buses: &Vec<BusBuilder>,
    messages: &Vec<MessageBuilder>,
) -> errors::Result<()> {
    let mut setset = MessageSetSet::new();
    for message in messages {
        setset.insert(message);
    }
    setset.merge_sets();

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::builder::{bus::BusBuilder, MessageBuilder, NetworkBuilder};

    use super::resolve_ids_filters_and_buses;

    #[test]
    fn idrp_test0() {
        let network_builder = NetworkBuilder::new();
        network_builder.create_node("secu");
        network_builder.create_node("becu");

        let secu_to_becu = network_builder.create_message("secu_to_becu");
        secu_to_becu.add_receiver("becu");
        secu_to_becu.add_transmitter("secu");

        let becu_to_secu = network_builder.create_message("becu_to_secu");
        becu_to_secu.add_receiver("secu");
        becu_to_secu.add_transmitter("becu");

        let becu_to_secu = network_builder.create_message("becu_to_secu_fixed_std");
        becu_to_secu.set_std_id(10);
        becu_to_secu.add_receiver("secu");
        becu_to_secu.add_transmitter("becu");

        let becu_to_secu = network_builder.create_message("becu_to_secu_fixed_bus");
        becu_to_secu.assign_bus("bus1");
        becu_to_secu.add_receiver("secu");
        becu_to_secu.add_transmitter("becu");

        // let becu_to_secu = network_builder.create_message("becu_to_secu_fixed_bus");
        // becu_to_secu.assign_bus("bus2");
        // becu_to_secu.add_receiver("secu");
        // becu_to_secu.add_transmitter("becu");
        //
        // let becu_to_secu = network_builder.create_message("becu_to_secu_fixed_ext");
        // becu_to_secu.set_ext_id(0x100);
        // becu_to_secu.add_receiver("secu");
        // becu_to_secu.add_transmitter("becu");

        let messages: Vec<MessageBuilder> = network_builder
            .0
            .borrow()
            .messages
            .borrow()
            .iter()
            .map(|m| m.clone())
            .collect();
        let buses: Vec<BusBuilder> = network_builder
            .0
            .borrow()
            .buses
            .borrow()
            .iter()
            .map(|b| b.clone())
            .collect();
        resolve_ids_filters_and_buses(&buses, &messages).unwrap();

        assert!(false);
    }
}
