use std::{
    cmp::Ordering,
    collections::{hash_map::DefaultHasher, HashSet, BTreeMap, BTreeSet, HashMap},
    hash::{Hash, Hasher},
    time::Duration,
};

use crate::{
    builder::{message_builder::{MessageBuilderUsage, MessageIdTemplate}, MessagePriority},
    config::{Type, TypeRef},
    errors,
};

use super::{bus::BusBuilder, MessageBuilder};

#[derive(Hash, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
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

#[derive(Hash, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
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

#[derive(Hash, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
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

#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
struct ReceiverSet {
    set: BTreeSet<String>,
    hash: u64,
}

impl ReceiverSet {
    fn new(message: &MessageBuilder) -> Self {
        let message_data = message.0.borrow();
        let receivers = &message_data.receivers;
        let mut set = BTreeSet::new();
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

#[derive(Hash, PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
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

    fn combine(a: &SetKey, b: &SetKey, options: &CombineOptions) -> Option<SetMerge> {
        if a.receiver_set != b.receiver_set {
            return None; // 2 sets can't be merged if their receive set is different
        }
        let mut assign_bus = None;
        let mut assign_std = false;
        let mut assign_ext = false;
        let mut assign_suffix = false;
        let type_assignment = match a.type_assignment {
            TypeAssignment::Std => {
                match b.type_assignment {
                    TypeAssignment::Std => TypeAssignment::Std,
                    TypeAssignment::Ext => return None, //can't merge sets of differnt types
                    TypeAssignment::Any => {
                        assign_std = true;
                        TypeAssignment::Std
                    }
                }
            }
            TypeAssignment::Ext => {
                match b.type_assignment {
                    TypeAssignment::Std => return None, //can't merge sets of differnt types
                    TypeAssignment::Ext => TypeAssignment::Ext,
                    TypeAssignment::Any if options.allow_ext => {
                        assign_ext = true;
                        TypeAssignment::Ext
                    }
                    TypeAssignment::Any => return None, // dont allow ext
                }
            }
            TypeAssignment::Any => {
                match b.type_assignment {
                    TypeAssignment::Std => {
                        assign_std = true;
                        TypeAssignment::Std
                    }
                    TypeAssignment::Ext if options.allow_ext => {
                        assign_ext = true;
                        TypeAssignment::Ext
                    }
                    TypeAssignment::Ext => return None, //dont allow ext
                    TypeAssignment::Any if options.allow_ext => TypeAssignment::Any,
                    TypeAssignment::Any => {
                        assign_std = true;
                        TypeAssignment::Std //prefer std if ext is not allowed
                    }
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
                    BusAssignment::Any => {
                        assign_bus = Some((a_bus_id, true));
                        BusAssignment::Bus { id: a_bus_id }
                    }
                }
            }
            BusAssignment::Any => match b.bus_assignment {
                BusAssignment::Bus { id: b_bus_id } => {
                    assign_bus = Some((b_bus_id, false));
                    BusAssignment::Bus { id: b_bus_id }
                }
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
                                    .overflowing_shl(29 - options.ext_suffix_len)
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
                    SuffixAssignment::None => {
                        assign_suffix = true;
                        SuffixAssignment::Suffix { value: suffix_a }
                    }
                }
            }
            SuffixAssignment::None => match b.suffix_assignment {
                SuffixAssignment::Suffix { value: suffix_b } => {
                    assign_suffix = true;
                    SuffixAssignment::Suffix { value: suffix_b }
                }
                SuffixAssignment::None => SuffixAssignment::None,
            },
        };

        Some(SetMerge {
            new_key: SetKey {
                receiver_set: a.receiver_set.clone(),
                bus_assignment,
                suffix_assignment,
                type_assignment,
            },
            assign_bus,
            assign_suffix,
            assign_std,
            assign_ext,
        })
    }
}

#[derive(Debug, Clone)]
struct SetMerge {
    new_key: SetKey,
    assign_bus: Option<(u32, bool)>,
    assign_std: bool,
    assign_ext: bool,
    assign_suffix: bool,
}

#[derive(Clone)]
struct MessageSet {
    key: SetKey,
    messages: Vec<MessageBuilder>,
    bus_load: f64,
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
            bus_load: 0.0,
        }
    }
    pub fn add_message(&mut self, message: &MessageBuilder, types: &Vec<TypeRef>) {
        // assert_eq!(SetKey::new(message), self.key);
        let std = match message.0.borrow().id {
            crate::builder::message_builder::MessageIdTemplate::StdId(_) => true,
            crate::builder::message_builder::MessageIdTemplate::ExtId(_) => false,
            crate::builder::message_builder::MessageIdTemplate::AnyStd(_) => true,
            crate::builder::message_builder::MessageIdTemplate::AnyExt(_) => false,
            crate::builder::message_builder::MessageIdTemplate::AnyAny(_) => false, // worst case!
        };
        // calc dlc!
        let dlc: usize = match &message.0.borrow().format {
            crate::builder::MessageFormat::Signals(signal_format) => signal_format
                .0
                .borrow()
                .0
                .iter()
                .map(|s| s.byte_offset() + s.size() as usize)
                .max()
                .unwrap_or(0)
                .into(),
            crate::builder::MessageFormat::Types(type_format) => {
                let mut acc: usize = 0;
                fn acc_dlc(ty: &Type) -> usize {
                    match ty {
                        Type::Primitive(signal_type) => signal_type.size() as usize,
                        Type::Struct {
                            name: _,
                            description: _,
                            attribs,
                            visibility: _,
                        } => {
                            let mut acc: usize = 0;
                            for (_, ty) in attribs {
                                acc += acc_dlc(ty as &Type);
                            }
                            acc
                        }
                        Type::Enum {
                            name: _,
                            description: _,
                            size,
                            entries: _,
                            visibility: _,
                        } => *size as usize,
                        Type::Array { len: _, ty: _ } => todo!(),
                    }
                }
                for (type_name, _) in &type_format.0.borrow().0 {
                    let ty = types.iter().find(|ty| &ty.name() == type_name);
                    match ty {
                        Some(ty) => acc += acc_dlc(ty as &Type),
                        None => {
                            println!("FIXME Please : can-config-rs : message_resolution_prototocol")
                        }
                    };
                }
                acc
            }
            crate::builder::MessageFormat::Empty => 0,
        };

        let bus_frame_load = if std {
            8 * dlc + 44 + (34 + 8 * dlc - 1) / 4
        } else {
            8 * dlc + 64 + (54 + 8 * dlc - 1) / 4
        };
        let interval = match &message.0.borrow().usage {
            MessageBuilderUsage::Stream(stream) => {
                //TODO actually get the correct interval
                stream.0.borrow().interval.0
            }
            MessageBuilderUsage::CommandReq(command) => command.0.borrow().expected_interval,
            MessageBuilderUsage::CommandResp(command) => command.0.borrow().expected_interval,
            MessageBuilderUsage::Configuration => Duration::from_secs(5),
            MessageBuilderUsage::External { interval } => {
                // intentionally really low because we might not be able to set this 
                // properly
                interval.unwrap_or(Duration::from_millis(50)) 
            }
            MessageBuilderUsage::Heartbeat => Duration::from_millis(100),
        };
        // println!("dlc = {dlc}");
        // println!("interval = {interval:?}");
        // println!("frame_len = {bus_frame_load}");
        let bus_load = (bus_frame_load as f64 * 1.0e9) / interval.as_nanos() as f64;
        // println!("bus_load = {bus_load}");
        self.bus_load += bus_load;

        self.messages.push(message.clone());
    }
}

struct MessageSetSet {
    sets: BTreeMap<SetKey, MessageSet>,
}
impl std::fmt::Debug for MessageSetSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.sets.values())
    }
}

impl MessageSetSet {
    pub fn new() -> Self {
        Self {
            sets: BTreeMap::new(),
        }
    }
    pub fn insert(&mut self, message: &MessageBuilder, types: &Vec<TypeRef>) {
        let key = SetKey::new(message);
        let set = self.sets.get_mut(&key);
        match set {
            Some(set) => {
                set.add_message(message, types);
            }
            None => {
                let mut set = MessageSet::new(key.clone());
                set.add_message(message, types);
                self.sets.insert(key, set);
            }
        }
    }

    pub fn merge_sets(&mut self, buses: &Vec<BusBuilder>, options: &CombineOptions) -> bool {
        // self.display_info(buses);

        let sets: Vec<MessageSet> = self.sets.values().map(|s| s.clone()).collect();

        let mut bus_cap: Vec<f64> = buses
            .iter()
            .map(|bus| bus.0.borrow().baudrate as f64)
            .collect();
        let set_count = self.sets.len();
        for (key, set) in &self.sets {
            match key.bus_assignment {
                BusAssignment::Bus { id } => bus_cap[id as usize] -= set.bus_load,
                BusAssignment::Any => (),
            }
        }

        // suffix collisions!

        #[derive(Debug, Clone)]
        struct Merge {
            i: usize,
            j: usize,
            merge_info: SetMerge,
        }

        let mut best_score = 0;
        let mut best_merge: Option<Merge> = None;

        for i in 0..set_count {
            for j in 0..set_count {
                if i == j {
                    continue;
                }
                let set_merge = SetKey::combine(&sets[i].key, &sets[j].key, &options);
                match set_merge {
                    Some(set_merge) => {
                        // evaulate !
                        let mut score = 0;
                        score += match &set_merge.assign_bus {
                            Some((assigned_bus, to_a)) => {
                                let additional_load = if *to_a {
                                    sets[j].bus_load
                                } else {
                                    sets[i].bus_load
                                };
                                if bus_cap[*assigned_bus as usize] < additional_load {
                                    // doesn't fit on the bus
                                    -1000
                                } else {
                                    let my_cap = bus_cap[*assigned_bus as usize];
                                    // * 3 / 2 to prefer this merges over any merges!
                                    let mut nicenes: u32 = bus_cap.len() as u32 * 3 / 2;
                                    for bus in &bus_cap {
                                        if *bus > my_cap {
                                            nicenes -= 1;
                                        }
                                    }
                                    nicenes as i32
                                }
                            }
                            None => {
                                let mut best_bus = 0;
                                let mut highest_cap = 0.0;
                                for i in 0..bus_cap.len() {
                                    if bus_cap[i] > highest_cap {
                                        highest_cap = bus_cap[i];
                                        best_bus = i;
                                    }
                                }
                                if bus_cap[best_bus] < sets[i].bus_load + sets[j].bus_load {
                                    panic!("it's impossible to find a bus balancing solution, try reducing the amount of messages");
                                };
                                bus_cap.len() as i32
                            }
                        };

                        if set_merge.assign_std {
                            score += (bus_cap.len() * 2) as i32;
                        }
                        if set_merge.assign_ext {
                            score += 0;
                        }
                        if set_merge.assign_suffix {
                            score += (bus_cap.len() / 2 + 1) as i32;
                        }

                        if score > best_score {
                            best_merge = Some(Merge {
                                i,
                                j,
                                merge_info: set_merge.clone(),
                            });
                            best_score = score;
                        }
                    }
                    None => (),
                }
            }
        }
        let Some(best_merge) = best_merge else {
            return false;
        };
        // apply merge!
        let key_a = &sets[best_merge.i].key;
        let mut set_a = self.sets.remove(&key_a).unwrap();
        let key_b = &sets[best_merge.j].key;
        let mut set_b = self.sets.remove(&key_b).unwrap();

        set_a.messages.append(&mut set_b.messages);
        let new_set = MessageSet {
            bus_load: set_a.bus_load + set_b.bus_load,
            key: best_merge.merge_info.new_key,
            messages: set_a.messages,
        };
        self.sets.insert(new_set.key.clone(), new_set);

        return true;
    }

    pub fn split_sets(
        &mut self,
        _buses: &Vec<BusBuilder>,
        types: &Vec<TypeRef>,
        _options: &CombineOptions,
    ) -> bool {
        let mut sets: BTreeMap<SetKey, MessageSet> = BTreeMap::new();
        let mut did_split = false;

        for (key, set) in &self.sets {
            match key.suffix_assignment {
                SuffixAssignment::Suffix { value: _ } => {
                    if set.messages.len() >= 127 {
                        did_split = true;
                        // Find all messages with assigned suffixes and collect them!
                        // Where does this suffix actually come from!

                        // compare suffixes for the len
                        // let suffix_mask = match set.key.type_assignment {
                        //     TypeAssignment::Std => {
                        //         (0xFFFFFFFF as u32)
                        //             .overflowing_shl(11 - options.std_suffix_len)
                        //             .0
                        //     }
                        //     TypeAssignment::Ext => {
                        //         (0xFFFFFFFF as u32)
                        //             .overflowing_shl(29 - options.ext_suffix_len)
                        //             .0
                        //     }
                        //     TypeAssignment::Any => {
                        //         panic!("if a suffix is specified the frame must have a type")
                        //     }
                        // };
                        let mut set_a = MessageSet {
                            bus_load: 0.0,
                            key: key.clone(),
                            messages: vec![],
                        };
                        let mut set_b = MessageSet {
                            bus_load: 0.0,
                            key: SetKey {
                                bus_assignment: key.bus_assignment.clone(),
                                receiver_set: key.receiver_set.clone(),
                                suffix_assignment: SuffixAssignment::None,
                                type_assignment: key.type_assignment.clone(),
                            },
                            messages: vec![],
                        };
                        // TODO implement priority based split

                        for msg in &set.messages {
                            let has_suffix = match &msg.0.borrow().id {
                                super::message_builder::MessageIdTemplate::StdId(_) => true,
                                super::message_builder::MessageIdTemplate::ExtId(_) => true,
                                _ => false,
                            };
                            if has_suffix {
                                set_a.add_message(msg, types);
                            } else {
                                if set_a.messages.len() > set_b.messages.len() {
                                    set_b.add_message(msg, types);
                                } else {
                                    set_a.add_message(msg, types);
                                }
                            }
                        }
                        sets.insert(set_a.key.clone(), set_a);
                        sets.insert(set_b.key.clone(), set_b);
                    } else {
                        // keep the set exactly the same!
                        // Alternativly we could check if the priority ordering is correct
                        // otherwise and allow us to split the set!
                        sets.insert(key.clone(), set.clone());
                    }
                }
                SuffixAssignment::None => {
                    // assign a new setcode! (key)
                    let key = key;
                    if set.messages.len() >= 127 {
                        did_split = true;
                        // split set into 2 sets with around the same priories
                        let mut set_a = MessageSet {
                            key: key.clone(),
                            messages: vec![],
                            bus_load: 0.0,
                        };
                        let mut set_b = MessageSet {
                            key: key.clone(),
                            messages: vec![],
                            bus_load: 0.0,
                        };

                        let sl_messages: Vec<&MessageBuilder> = set
                            .messages
                            .iter()
                            .filter(|m| match &m.0.borrow().id {
                                super::message_builder::MessageIdTemplate::StdId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::ExtId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::AnyStd(prio) => {
                                    prio == &MessagePriority::SuperLow
                                }
                                super::message_builder::MessageIdTemplate::AnyExt(prio) => {
                                    prio == &MessagePriority::SuperLow
                                }
                                super::message_builder::MessageIdTemplate::AnyAny(prio) => {
                                    prio == &MessagePriority::SuperLow
                                }
                            })
                            .collect();

                        for msg in &sl_messages[0..sl_messages.len() / 2] {
                            set_a.add_message(msg, types);
                        }
                        for msg in &sl_messages[sl_messages.len()..] {
                            set_b.add_message(msg, types);
                        }

                        let low_messages: Vec<&MessageBuilder> = set
                            .messages
                            .iter()
                            .filter(|m| match &m.0.borrow().id {
                                super::message_builder::MessageIdTemplate::StdId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::ExtId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::AnyStd(prio) => {
                                    prio == &MessagePriority::Low
                                }
                                super::message_builder::MessageIdTemplate::AnyExt(prio) => {
                                    prio == &MessagePriority::Low
                                }
                                super::message_builder::MessageIdTemplate::AnyAny(prio) => {
                                    prio == &MessagePriority::Low
                                }
                            })
                            .collect();

                        for msg in &low_messages[0..low_messages.len() / 2] {
                            set_a.add_message(msg, types);
                        }
                        for msg in &sl_messages[sl_messages.len()..] {
                            set_b.add_message(msg, types);
                        }

                        let normal_messages: Vec<&MessageBuilder> = set
                            .messages
                            .iter()
                            .filter(|m| match &m.0.borrow().id {
                                super::message_builder::MessageIdTemplate::StdId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::ExtId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::AnyStd(prio) => {
                                    prio == &MessagePriority::Normal
                                }
                                super::message_builder::MessageIdTemplate::AnyExt(prio) => {
                                    prio == &MessagePriority::Normal
                                }
                                super::message_builder::MessageIdTemplate::AnyAny(prio) => {
                                    prio == &MessagePriority::Normal
                                }
                            })
                            .collect();

                        for msg in &normal_messages[0..normal_messages.len() / 2] {
                            set_a.add_message(msg, types);
                        }
                        for msg in &sl_messages[sl_messages.len()..] {
                            set_b.add_message(msg, types);
                        }

                        let high_messages: Vec<&MessageBuilder> = set
                            .messages
                            .iter()
                            .filter(|m| match &m.0.borrow().id {
                                super::message_builder::MessageIdTemplate::StdId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::ExtId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::AnyStd(prio) => {
                                    prio == &MessagePriority::High
                                }
                                super::message_builder::MessageIdTemplate::AnyExt(prio) => {
                                    prio == &MessagePriority::High
                                }
                                super::message_builder::MessageIdTemplate::AnyAny(prio) => {
                                    prio == &MessagePriority::High
                                }
                            })
                            .collect();

                        for msg in &high_messages[0..high_messages.len() / 2] {
                            set_a.add_message(msg, types);
                        }
                        for msg in &sl_messages[sl_messages.len()..] {
                            set_b.add_message(msg, types);
                        }

                        let realtime_messages: Vec<&MessageBuilder> = set
                            .messages
                            .iter()
                            .filter(|m| match &m.0.borrow().id {
                                super::message_builder::MessageIdTemplate::StdId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::ExtId(_) => {
                                    panic!("We shoudn't be here!")
                                }
                                super::message_builder::MessageIdTemplate::AnyStd(prio) => {
                                    prio == &MessagePriority::Realtime
                                }
                                super::message_builder::MessageIdTemplate::AnyExt(prio) => {
                                    prio == &MessagePriority::Realtime
                                }
                                super::message_builder::MessageIdTemplate::AnyAny(prio) => {
                                    prio == &MessagePriority::Realtime
                                }
                            })
                            .collect();

                        for msg in &realtime_messages[0..realtime_messages.len() / 2] {
                            set_a.add_message(msg, types);
                        }
                        for msg in &sl_messages[sl_messages.len()..] {
                            set_b.add_message(msg, types);
                        }
                        sets.insert(key.clone(), set_a);
                        sets.insert(key.clone(), set_b);
                    } else {
                        sets.insert(key.clone(), set.clone());
                    }
                }
            }
        }
        self.sets = sets;
        return did_split;
    }

    pub fn fix_sets(&mut self, buses: &Vec<BusBuilder>, options: &CombineOptions) {
        let mut sets: BTreeMap<SetKey, MessageSet> = BTreeMap::new();

        let mut std_setcodes: Vec<u32> = vec![];
        let mut ext_setcodes: Vec<u32> = vec![];

        for (key, _) in &self.sets {
            match key.type_assignment {
                TypeAssignment::Std => match key.suffix_assignment {
                    SuffixAssignment::Suffix { value } => std_setcodes.push(value),
                    SuffixAssignment::None => (),
                },
                TypeAssignment::Ext => match key.suffix_assignment {
                    SuffixAssignment::Suffix { value } => ext_setcodes.push(value),
                    SuffixAssignment::None => (),
                },
                TypeAssignment::Any => (),
            }
        }

        let mut bus_cap: Vec<f64> = buses
            .iter()
            .map(|bus| bus.0.borrow().baudrate as f64)
            .collect();
        for (key, set) in &self.sets {
            match key.bus_assignment {
                BusAssignment::Bus { id } => bus_cap[id as usize] -= set.bus_load,
                BusAssignment::Any => (),
            }
        }
        let mut sets_vec: Vec<&MessageSet> = self.sets.iter().map(|(_, set)| set).collect();
        sets_vec.sort_by(|a, b| {
            if a.bus_load < b.bus_load {
                Ordering::Greater
            } else if a.bus_load > b.bus_load {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });

        for set in sets_vec {
            let key = &set.key;

            let type_assignment = match key.type_assignment {
                TypeAssignment::Std => key.type_assignment.clone(),
                TypeAssignment::Ext => key.type_assignment.clone(),
                TypeAssignment::Any => {
                    // try to find a std set assignment!
                    if std_setcodes.len() >= 16 && ext_setcodes.len() <= 255 {
                        TypeAssignment::Ext
                    } else if std_setcodes.len() < 16 {
                        TypeAssignment::Std
                    } else {
                        panic!()
                    }
                }
            };

            let suffix_mask = match type_assignment {
                TypeAssignment::Std => {
                    (0xFFFFFFFF as u32)
                        .overflowing_shl(11 - options.std_suffix_len)
                        .0
                }
                TypeAssignment::Ext => {
                    (0xFFFFFFFF as u32)
                        .overflowing_shl(29 - options.ext_suffix_len)
                        .0
                }
                TypeAssignment::Any => {
                    panic!("wtf")
                }
            };
            let suffix_assignment = match key.suffix_assignment {
                SuffixAssignment::Suffix { value } => SuffixAssignment::Suffix {
                    value: value & suffix_mask,
                },
                SuffixAssignment::None => match type_assignment {
                    TypeAssignment::Std => {
                        let mut x: Option<SuffixAssignment> = None;
                        for i in 0..16 {
                            let setcode = i << 7;
                            let suffix = if std_setcodes.contains(&setcode) {
                                continue;
                            } else {
                                std_setcodes.push(setcode);
                                SuffixAssignment::Suffix { value: setcode }
                            };
                            x = Some(suffix);
                            break;
                        }
                        x
                    }
                    TypeAssignment::Ext => {
                        let mut x: Option<SuffixAssignment> = None;
                        for i in 0..16 {
                            let setcode = i << 7;
                            let suffix = if std_setcodes.contains(&setcode) {
                                continue;
                            } else {
                                ext_setcodes.push(setcode);
                                SuffixAssignment::Suffix { value: setcode }
                            };
                            x = Some(suffix);
                            break;
                        }
                        x
                    }
                    TypeAssignment::Any => panic!(),
                }
                .expect("I thought this should never happen, well i guess i was wrong"),
            };
            let bus_assignment = match key.bus_assignment {
                BusAssignment::Bus { id } => BusAssignment::Bus { id },
                BusAssignment::Any => {
                    // search for the most empty bus
                    let mut best_bus = 0;
                    let mut best_cap = 0.0;
                    for i in 0..bus_cap.len() {
                        // println!("CAP = {}", bus_cap[i]);
                        if bus_cap[i] > best_cap {
                            best_cap = bus_cap[i];
                            best_bus = i;
                        }
                    }
                    if bus_cap[best_bus] < set.bus_load {
                        panic!("ohhh this is really bad try to reduce the number of messages if not possible iam really really sorry. PS Karl");
                    }
                    bus_cap[best_bus] -= set.bus_load;
                    BusAssignment::Bus {
                        id: best_bus as u32,
                    }
                }
            };
            let BusAssignment::Bus{ id : bus_id} = bus_assignment else {panic!();};
            let key = SetKey {
                bus_assignment,
                type_assignment,
                suffix_assignment,
                receiver_set: key.receiver_set.clone(),
            };
            for msg in &set.messages {
                let bus = buses.iter().find(|b| b.0.borrow().id == bus_id).unwrap();
                let bus_name = bus.0.borrow().name.clone();
                msg.assign_bus(&bus_name);
            }
            sets.insert(
                key.clone(),
                MessageSet {
                    key,
                    bus_load: set.bus_load,
                    messages: set.messages.clone(),
                },
            );
        }

        // assign ids
        for (key, set) in &mut sets {
            let suffix_mask = match key.type_assignment {
                TypeAssignment::Std => {
                    (0xFFFFFFFF as u32)
                        .overflowing_shl(11 - options.std_suffix_len)
                        .0
                }
                TypeAssignment::Ext => {
                    (0xFFFFFFFF as u32)
                        .overflowing_shl(29 - options.ext_suffix_len)
                        .0
                }
                TypeAssignment::Any => {
                    panic!("wtf")
                }
            };
            let SuffixAssignment::Suffix { value: suffix } = key.suffix_assignment else {
                panic!();
            };
            let suffix = suffix & suffix_mask;

            let set_size = 127;
            let mut assigned_ids: HashSet<i32> = HashSet::new();
            let mut next_ids: Vec<i32> = vec![];
            let id_sep = set_size / MessagePriority::count();
            let mut curr = set_size % MessagePriority::count();
            for _ in 0..MessagePriority::count() {
                curr += id_sep;
                next_ids.push(curr as i32);
            }
            let mut to_assign: Vec<(MessageBuilder, i32)> = vec![];
            for msg in &set.messages {
                match &msg.0.borrow().id {
                    super::message_builder::MessageIdTemplate::StdId(id) => {
                        assigned_ids.insert(*id as i32);
                    }
                    super::message_builder::MessageIdTemplate::ExtId(id) => {
                        assigned_ids.insert(*id as i32);
                    }
                    super::message_builder::MessageIdTemplate::AnyStd(prio) => {
                        to_assign.push((msg.clone(), prio.to_u32() as i32));
                    }
                    super::message_builder::MessageIdTemplate::AnyExt(prio) => {
                        to_assign.push((msg.clone(), prio.to_u32() as i32));
                    }
                    super::message_builder::MessageIdTemplate::AnyAny(prio) => {
                        to_assign.push((msg.clone(), prio.to_u32() as i32));
                    }
                }
            }
            to_assign.sort_by_key(|(_, prio)| *prio);
            
            for (msg, prio) in to_assign {
                loop {
                    let try_id = next_ids[prio as usize];
                    if try_id < 0 {
                        panic!();
                    }
                    if assigned_ids.contains(&try_id) {
                        if try_id == 0 {
                            panic!("Failed to find a id assignment");
                        }
                        let next_try = try_id - 1;
                        next_ids[prio as usize] = next_try;
                    } else {
                        match key.type_assignment {
                            TypeAssignment::Std => msg.set_std_id(try_id as u32 | suffix),
                            TypeAssignment::Ext => msg.set_ext_id(try_id as u32 | suffix),
                            TypeAssignment::Any => panic!("We shoudn't be here!"),
                        };
                        let next_try = try_id - 1;
                        next_ids[prio as usize] = next_try;
                        break;
                    }
                }
            }
        }

        self.sets = sets;
    }

    pub fn display_info(&self, buses: &Vec<BusBuilder>, priority_map : HashMap<String, MessageIdTemplate>) {
        // let mut i = 0;
        // for (key, set) in &self.sets {
        //     println!("==========Set {i}===========");
        //     println!("-receivers : {:?}", key.receiver_set.set);
        //     println!("-bus       : {:?}", key.bus_assignment);
        //     println!("-load      : {:?}", set.bus_load);
        //     println!("-type      : {:?}", key.type_assignment);
        //     match key.suffix_assignment {
        //         SuffixAssignment::Suffix { value } => println!("-setcode   : 0x{value:X}"),
        //         SuffixAssignment::None => println!("-setcode   : ?"),
        //     }
        //     println!("-messages  : {}", set.messages.len());
        //     for msg in &set.messages {
        //         print!("--{} : ", msg.0.borrow().name);
        //         match msg.0.borrow().id {
        //             crate::builder::message_builder::MessageIdTemplate::StdId(id) => println!("0x{id:X}"),
        //             crate::builder::message_builder::MessageIdTemplate::ExtId(id) => println!("0x{id:X}"),
        //             crate::builder::message_builder::MessageIdTemplate::AnyStd(prio) => println!("p{}", prio.to_u32()),
        //             crate::builder::message_builder::MessageIdTemplate::AnyExt(prio) => println!("p{}", prio.to_u32()),
        //             crate::builder::message_builder::MessageIdTemplate::AnyAny(prio) => println!("p{}", prio.to_u32()),
        //         }
        //     }
        //
        //     i += 1;
        // }
        let mut messages : Vec<MessageBuilder>= self.sets.iter().map(|set| set.1.messages.clone()).flatten().collect();
        assert_eq!(messages.len(), priority_map.len());
        messages.sort_by(|a,b| {
            match a.0.borrow().id {
                crate::builder::message_builder::MessageIdTemplate::StdId(aid) => {
                    match b.0.borrow().id {
                        crate::builder::message_builder::MessageIdTemplate::StdId(bid) => {
                            if aid < bid {
                                Ordering::Less
                            }else {
                                Ordering::Greater
                            }
                        }
                        crate::builder::message_builder::MessageIdTemplate::ExtId(_) => Ordering::Less,
                        _ => panic!(),
                    }
                },
                crate::builder::message_builder::MessageIdTemplate::ExtId(aid) => {
                    match b.0.borrow().id {
                        crate::builder::message_builder::MessageIdTemplate::StdId(_) => Ordering::Greater,
                        crate::builder::message_builder::MessageIdTemplate::ExtId(bid) => {
                            if aid < bid {
                                Ordering::Less
                            }else {
                                Ordering::Greater
                            }
                        }
                        _ => panic!(),
                    }
                }
                _ => panic!(),
            }
        });
        println!("===========ID-ASSIGNMENT=====================");
        for msg in &messages {
            let id_str = match msg.0.borrow().id {
                MessageIdTemplate::StdId(id) => format!("0x{id:X}"),
                MessageIdTemplate::ExtId(id) => format!("0x{id:X}x"),
                _ => panic!(),
            };

            let priority = match priority_map.get(&msg.0.borrow().name) {
                Some(id_temp) => match id_temp {
                    MessageIdTemplate::StdId(_) => "STD_FIXED".to_owned(),
                    MessageIdTemplate::ExtId(_) => "EXT_FIXED".to_owned(),
                    MessageIdTemplate::AnyStd(prio) => {
                        match prio {
                            MessagePriority::Realtime => "STD-REALTIME".to_owned(),
                            MessagePriority::High => "STD-HIGH".to_owned(),
                            MessagePriority::Normal => "STD-NORMAL".to_owned(),
                            MessagePriority::Low => "STD-LOW".to_owned(),
                            MessagePriority::SuperLow => "STD-SUPER-LOW".to_owned(),
                        }
                    }
                    MessageIdTemplate::AnyExt(prio) => {
                        match prio {
                            MessagePriority::Realtime => "EXT-REALTIME".to_owned(),
                            MessagePriority::High => "EXT-HIGH".to_owned(),
                            MessagePriority::Normal => "EXT-NORMAL".to_owned(),
                            MessagePriority::Low => "EXT-LOW".to_owned(),
                            MessagePriority::SuperLow => "EXT-SUPER-LOW".to_owned(),
                        }
                    }
                    MessageIdTemplate::AnyAny(prio) => {
                        match prio {
                            MessagePriority::Realtime => "ANY-REALTIME".to_owned(),
                            MessagePriority::High => "ANY-HIGH".to_owned(),
                            MessagePriority::Normal => "ANY-NORMAL".to_owned(),
                            MessagePriority::Low => "ANY-LOW".to_owned(),
                            MessagePriority::SuperLow => "ANY-SUPER-LOW".to_owned(),
                        }
                    }
                }
                None => panic!("droped a message somewhere!"),
            };

            println!("{: <35} {priority: <20} -> {id_str: <6}", msg.0.borrow().name);
        }
        
        println!("============ESTIMATED-BUS-LOADS==============");
        for bus in buses {
            let mut bus_load = 0.0;
            for (key, set) in &self.sets {
                match key.bus_assignment {
                    BusAssignment::Bus { id } => {
                        if bus.0.borrow().id == id {
                            bus_load += set.bus_load;
                        }
                    }
                    BusAssignment::Any => (),
                }
            }
            let bus_cap = bus.0.borrow().baudrate;
            // println!("-load  : {bus_load} / {bus_cap}");

            println!("-bus {} : with -> {:.2}/{} kib/s = {:.4}%", bus.0.borrow().id, bus_load, bus_cap as f64 / 1000.0, (bus_load / bus_cap as f64) * 100.0);
        }
        println!("=============================================");
    }
}

pub fn resolve_ids_filters_and_buses(
    buses: &Vec<BusBuilder>,
    messages: &Vec<MessageBuilder>,
    types: &Vec<TypeRef>,
) -> errors::Result<()> {

    // store the id assignments of the messages before modifying them 
    // only required for debug output or debug info.
    // iam aware that this is horrible, but i realized to late that builders
    // are mutable datatypes.
    let mut priority_map : HashMap<String, MessageIdTemplate> = HashMap::new();
    for msg in messages {
        priority_map.insert(msg.0.borrow().name.clone(), msg.0.borrow().id.clone());
    }

    let mut setset = MessageSetSet::new();
    for message in messages {
        setset.insert(message, types);
    }

    // Dont change me the code is written to only work with those values!
    // i know kind of stupid
    let options = CombineOptions {
        allow_ext: true,
        std_suffix_len: 4,
        ext_suffix_len: 8,
    };

    // setset.display_info(buses);

    // merge as many sets as possible
    while setset.merge_sets(buses, &options) {}

    // split sets so that no set is bigger than 127 messages!
    // in the based base case while accounting for priority
    while setset.split_sets(buses, types, &options) {}

    setset.fix_sets(buses, &options);

    setset.display_info(buses, priority_map);

    // Log some cool stats

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::builder::{bus::BusBuilder, MessageBuilder, MessagePriority, NetworkBuilder};

    use super::resolve_ids_filters_and_buses;

    #[test]
    fn idrp_test0() {
        let network_builder = NetworkBuilder::new();
        network_builder.create_node("secu");
        network_builder.create_node("becu");
        network_builder.create_bus("can1", Some(1000000));
        network_builder.create_bus("can2", Some(1000000));

        let secu_to_becu = network_builder.create_message("secu_to_becu", None);
        secu_to_becu.set_any_std_id(MessagePriority::Low);
        secu_to_becu.add_receiver("becu");
        secu_to_becu.add_transmitter("secu");

        let becu_to_secu = network_builder.create_message("becu_to_secu_2", None);
        becu_to_secu.set_std_id(0x500);
        becu_to_secu.add_receiver("secu");
        becu_to_secu.add_transmitter("becu");

        let becu_to_secu = network_builder.create_message("becu_to_secu_3", None);
        becu_to_secu.set_std_id(0x200);
        becu_to_secu.add_receiver("secu");
        becu_to_secu.add_transmitter("becu");

        for i in 0..50 {
            let becu_to_secu =
                network_builder.create_message(&format!("becu_to_secu_low_{i}"), Some(Duration::from_millis(1000)));
            becu_to_secu.set_any_std_id(MessagePriority::Low);
            becu_to_secu.add_receiver("secu");
            becu_to_secu.add_transmitter("becu");
        }
        for i in 0..50 {
            let becu_to_secu =
                network_builder.create_message(&format!("becu_to_secu_normal_{i}"), Some(Duration::from_millis(500)));
            becu_to_secu.set_any_std_id(MessagePriority::Normal);
            becu_to_secu.add_receiver("secu");
            becu_to_secu.add_transmitter("becu");
        }
        for i in 0..50 {
            let becu_to_secu =
                network_builder.create_message(&format!("becu_to_secu_high_{i}"), Some(Duration::from_millis(100)));
            becu_to_secu.set_any_std_id(MessagePriority::High);
            becu_to_secu.add_receiver("secu");
            becu_to_secu.add_transmitter("becu");
        }
        for i in 0..50 {
            let becu_to_secu =
                network_builder.create_message(&format!("becu_to_secu_realtime_{i}"), Some(Duration::from_millis(50)));
            becu_to_secu.set_any_std_id(MessagePriority::Realtime);
            becu_to_secu.add_receiver("secu");
            becu_to_secu.add_transmitter("becu");
        }


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
        resolve_ids_filters_and_buses(&buses, &messages, &vec![]).unwrap();
        assert!(false);
    }

    #[test]
    fn idrp_test1() {

    }
}
