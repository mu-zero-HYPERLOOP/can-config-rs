use std::{cell::RefCell, rc::Rc, time::Duration};

use crate::{
    config::{Type, TypeRef},
    errors,
};

use super::{
    bus::BusBuilder,
    message_builder::{self, MessageBuilderUsage},
    MessageBuilder, MessagePriority, NodeBuilder,
};

const MIN_SETCODE_LEN: u32 = 3;

const STD_FRAME_COUNT: u32 = 0x7FF;
const STD_PRIORITY_RANGE: u32 = STD_FRAME_COUNT / MessagePriority::count() as u32;
const EXT_FRAME_COUNT: u32 = 0x1FFFFFFF;
const EXT_PRIORITY_RANGE: u32 = EXT_FRAME_COUNT / MessagePriority::count() as u32;

const EXTERNAL_INTERVAL_MS: u64 = 50;
const HEARTBEAT_INTERVAL_MS: u64 = 100;

const ASSIGN_EXT_WEIGHT: u32 = 20;
const ASSIGN_STD_WEIGHT: u32 = 10;
const CREATE_SET_WEIGHT: u32 = 15;


struct ReceiverSetCommit {
    message: MessageBuilder,
    prio_u32: u32,
    expected_load: f64,
    assign_ext: bool,
    assign_std: bool,
    receiver_set: ReceiverSet,
}

impl ReceiverSetCommit{
    pub fn apply(&self) {
        self.receiver_set.0.borrow_mut().messages[self.prio_u32 as usize].push(self.message.clone());
    }
}

#[derive(Clone)]
struct ReceiverSet(Rc<RefCell<ReceiverSetData>>);

struct ReceiverSetData {
    setcode: u32,
    setcode_len: u32,
    ide: bool,
    messages: Vec<Vec<MessageBuilder>>,
    receivers: Vec<String>,
}


impl ReceiverSet {
    pub fn new(setcode: u32, setcode_len: u32, ide: bool, receivers: Vec<NodeBuilder>) -> Self {
        let mut receivers: Vec<String> = receivers
            .iter()
            .map(|r| r.0.borrow().name.clone())
            .collect();
        receivers.sort();
        ReceiverSet(Rc::new(RefCell::new(ReceiverSetData {
            setcode,
            setcode_len,
            ide,
            messages: vec![Vec::<MessageBuilder>::new(); MessagePriority::count()],
            receivers,
        })))
    }
    pub fn message_buckets(&self) -> Vec<Vec<MessageBuilder>> {
        self.0.borrow().messages.clone()
    }
    pub fn setcode(&self) -> u32 {
        self.0.borrow().setcode
    }
    pub fn setcode_len(&self) -> u32 {
        self.0.borrow().setcode_len
    }
    pub fn ide(&self) -> bool {
        self.0.borrow().ide
    }
    pub fn try_commit(
        &self,
        message: &MessageBuilder,
        types: &Vec<TypeRef>,
    ) -> Option<ReceiverSetCommit> {
        // matching ide flag!
        let (assign_ext, assign_std, matching_ide) = match message.0.borrow().id {
            super::message_builder::MessageIdTemplate::StdId(_) => (false, false, !self.ide()),
            super::message_builder::MessageIdTemplate::ExtId(_) => (false, false, self.ide()),
            message_builder::MessageIdTemplate::AnyStd(_) => (false, false, !self.ide()),
            message_builder::MessageIdTemplate::AnyExt(_) => (false, false, self.ide()),
            message_builder::MessageIdTemplate::AnyAny(_) => {
                if self.ide() {
                    (false, true, true)
                } else {
                    (true, false, true)
                }
            }
        };
        if !matching_ide {
            return None;
        }

        // matching setcode!
        let matching_setcode = match message.0.borrow().id {
            super::message_builder::MessageIdTemplate::ExtId(id)
            | super::message_builder::MessageIdTemplate::StdId(id) => {
                id & ((0xFFFFFFFF as u32)
                    .overflowing_shl(32 - self.setcode_len())
                    .0)
                    == self.setcode()
            }
            _ => true,
        };

        if !matching_setcode {
            return None;
        }

        let dlc = match &message.0.borrow().format {
            super::MessageFormat::Signals(format) => format
                .0
                .borrow()
                .0
                .iter()
                .map(|s| s.byte_offset() + s.size() as usize)
                .max()
                .unwrap_or(0),
            super::MessageFormat::Types(type_format) => {
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
            super::MessageFormat::Empty => 0,
        };

        let bus_frame_load = if self.ide() {
            8 * dlc + 64 + (54 + 8 * dlc - 1) / 4
        } else {
            8 * dlc + 44 + (34 + 8 * dlc - 1) / 4
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
                interval.unwrap_or(Duration::from_millis(EXTERNAL_INTERVAL_MS))
            }
            MessageBuilderUsage::Heartbeat => Duration::from_millis(HEARTBEAT_INTERVAL_MS),
        };
        let expected_load = (bus_frame_load as f64 * 1.0e9) / interval.as_nanos() as f64;

        assert!(!self
            .message_buckets()
            .iter()
            .flatten()
            .any(|m| m.0.borrow().name == message.0.borrow().name));

        match message.0.borrow().id {
            message_builder::MessageIdTemplate::StdId(id) => {
                let prio_u32 = id / STD_PRIORITY_RANGE.min(MessagePriority::count() as u32 - 1);
                let allocated = self.message_buckets()[prio_u32 as usize].len();
                if allocated as u32 >= STD_PRIORITY_RANGE {
                    return None;
                }
                return Some(ReceiverSetCommit {
                    message: message.clone(),
                    prio_u32,
                    expected_load,
                    assign_ext,
                    assign_std,
                    receiver_set: self.clone(),
                });
            }
            message_builder::MessageIdTemplate::ExtId(id) => {
                let prio_u32 = id / EXT_PRIORITY_RANGE.min(MessagePriority::count() as u32 - 1);
                let allocated = self.message_buckets()[prio_u32 as usize].len();
                if allocated as u32 >= EXT_PRIORITY_RANGE {
                    return None;
                }
                return Some(ReceiverSetCommit {
                    message: message.clone(),
                    prio_u32,
                    expected_load,
                    assign_ext,
                    assign_std,
                    receiver_set: self.clone(),
                });
            }
            message_builder::MessageIdTemplate::AnyStd(prio)
            | message_builder::MessageIdTemplate::AnyExt(prio)
            | message_builder::MessageIdTemplate::AnyAny(prio) => {
                let range = if self.ide() {
                    EXT_PRIORITY_RANGE
                } else {
                    STD_PRIORITY_RANGE
                };
                let prio_u32 = prio.to_u32();
                let allocated = self.message_buckets()[prio_u32 as usize].len();
                if allocated as u32 >= range {
                    return None;
                }
                return Some(ReceiverSetCommit {
                    message: message.clone(),
                    prio_u32,
                    expected_load,
                    assign_ext,
                    assign_std,
                    receiver_set: self.clone(),
                });
            }
        }
    }
}

struct BusSetCommit {
    receiver_set_commit: ReceiverSetCommit,
    bus_set: BusSet,
    score: u32,
    new_set: Option<ReceiverSet>,
}

impl BusSetCommit {
    pub fn apply(&self) {
        match &self.new_set {
            Some(new_set) => {
                let setcode_pos = self
                    .bus_set
                    .0
                    .borrow()
                    .free_setcodes
                    .iter()
                    .position(|setcode| *setcode == new_set.setcode())
                    .expect("setcode is not free");
                self.bus_set.0.borrow_mut().free_setcodes.remove(setcode_pos);
                self.bus_set.0.borrow_mut().receiver_sets.push(new_set.clone());
                let load = self.receiver_set_commit.expected_load;
                self.bus_set.0.borrow_mut().bus_load += load;
                self.receiver_set_commit.apply();
            }
            None => (),
        }
    }
}

#[derive(Clone)]
struct BusSet(Rc<RefCell<BusSetData>>);

struct BusSetData {
    bus_id: u32,
    receiver_sets: Vec<ReceiverSet>,
    free_setcodes: Vec<u32>,
    setcode_len: u32,
    bus_load: f64,
}

impl BusSet {
    pub fn new(bus_id: u32) -> Self {
        let setcode_len: u32 = 1;
        let mut free_setcodes = vec![];
        for setcode in 0..(2 as u32).pow(setcode_len) {
            free_setcodes.push(setcode);
        }
        BusSet(Rc::new(RefCell::new(BusSetData {
            bus_id,
            free_setcodes,
            setcode_len,
            receiver_sets: vec![],
            bus_load: 0.0,
        })))
    }
    pub fn busload(&self) -> f64 {
        self.0.borrow().bus_load
    }
    pub fn receiver_sets(&self) -> Vec<ReceiverSet> {
        self.0.borrow().receiver_sets.clone()
    }
    pub fn next_setcode(&self) -> Option<u32> {
        self.0.borrow().free_setcodes.first().cloned()
    }
    pub fn setcode_len(&self) -> u32 {
        self.0.borrow().setcode_len
    }
    pub fn free_setcode_count(&self) -> u32 {
        self.0.borrow().free_setcodes.len() as u32
    }
    pub fn setcode_count(&self) -> u32 {
        (2 as u32).pow(self.setcode_len())
    }

    pub fn increment_setcode_len(&self, types : &Vec<TypeRef>) {
        // keep current sets!

        // reinitalize bus set with incremented setcode_len


    }

    pub fn try_commit(
        &self,
        message: &MessageBuilder,
        types: &Vec<TypeRef>,
    ) -> Option<BusSetCommit> {
        let mut extending_commits: Vec<ReceiverSetCommit> = vec![];
        // try insertion into existing sets!
        for receiver_set in &mut self.receiver_sets() {
            match receiver_set.try_commit(message, types) {
                Some(commit) => {
                    extending_commits.push(commit);
                }
                None => continue,
            }
        }
        let mut constructing_commits: Vec<(ReceiverSet, ReceiverSetCommit)> = vec![];
        let empty_setcode = self.next_setcode();
        match empty_setcode {
            Some(setcode) => {
                let ide = match message.0.borrow().id {
                    message_builder::MessageIdTemplate::StdId(_) => Some(false),
                    message_builder::MessageIdTemplate::ExtId(_) => Some(true),
                    message_builder::MessageIdTemplate::AnyStd(_) => Some(false),
                    message_builder::MessageIdTemplate::AnyExt(_) => Some(true),
                    message_builder::MessageIdTemplate::AnyAny(_) => None,
                };
                match ide {
                    Some(ide) => {
                        let receiver_set = ReceiverSet::new(
                            setcode,
                            self.setcode_len(),
                            ide,
                            message.0.borrow().receivers.clone().clone(),
                        );
                        let receiver_set_commit = receiver_set.try_commit(message, types);
                        match receiver_set_commit {
                            Some(commit) => constructing_commits.push((receiver_set, commit)),
                            None => (),
                        }
                    }
                    None => {
                        // create std set.
                        let receiver_set = ReceiverSet::new(
                            setcode,
                            self.setcode_len(),
                            false,
                            message.0.borrow().receivers.clone().clone(),
                        );
                        let receiver_set_commit = receiver_set.try_commit(message, types);
                        match receiver_set_commit {
                            Some(commit) => constructing_commits.push((receiver_set, commit)),
                            None => (),
                        }

                        // create ext set.
                        let receiver_set = ReceiverSet::new(
                            setcode,
                            self.setcode_len(),
                            true,
                            message.0.borrow().receivers.clone().clone(),
                        );
                        let receiver_set_commit = receiver_set.try_commit(message, types);
                        match receiver_set_commit {
                            Some(commit) => constructing_commits.push((receiver_set, commit)),
                            None => (),
                        }
                    }
                }
            }
            None => (),
        }

        let mut bus_set_commit: Vec<BusSetCommit> = vec![];
        for receiver_commit in extending_commits {
            let mut score = 0;
            if receiver_commit.assign_ext {
                score += ASSIGN_EXT_WEIGHT;
            }
            if receiver_commit.assign_std {
                score += ASSIGN_STD_WEIGHT;
            }

            bus_set_commit.push(BusSetCommit {
                bus_set: self.clone(),
                receiver_set_commit: receiver_commit,
                score,
                new_set: None,
            })
        }

        for (receiver_set, receiver_commit) in constructing_commits {
            let mut score = 0;
            if receiver_commit.assign_ext {
                score += ASSIGN_EXT_WEIGHT;
            }
            if receiver_commit.assign_std {
                score += ASSIGN_STD_WEIGHT;
            }
            let free_setcodes = self.free_setcode_count();
            score +=
                CREATE_SET_WEIGHT - ((free_setcodes * CREATE_SET_WEIGHT) / self.setcode_count());

            bus_set_commit.push(BusSetCommit {
                bus_set: self.clone(),
                receiver_set_commit: receiver_commit,
                score,
                new_set: Some(receiver_set),
            })
        }
        bus_set_commit.into_iter().max_by_key(|c| c.score)
    }
}

struct NetworkSet {
    bus_sets: Vec<BusSet>,
}

impl NetworkSet {
    pub fn new(buses: &Vec<BusBuilder>) -> Self {
        let bus_sets = buses
            .iter()
            .map(|bus| BusSet::new(bus.0.borrow().id))
            .collect();
        Self { bus_sets }
    }
    pub fn insert(&mut self, message: &MessageBuilder, types: &Vec<TypeRef>) {
        let mut bus_commits: Vec<BusSetCommit> = vec![];
        for bus_set in &self.bus_sets {
            let commit = bus_set.try_commit(message, types);
            match commit {
                Some(commit) => bus_commits.push(commit),
                None => (),
            }
        }
        let best_commit = bus_commits.iter().max_by_key(|c| c.score);
        match best_commit {
            Some(best_commit) => best_commit.apply(),
            None => {
                // retry! by incrementing a setcode len of a bus!
            }
        }
    }
}

pub fn resolve_ids_filters_and_buses(
    buses: &Vec<BusBuilder>,
    messages: &Vec<MessageBuilder>,
    types: &Vec<TypeRef>,
) -> errors::Result<()> {
    let mut network_set = NetworkSet::new(buses);
    
    for message in messages {
        match message.0.borrow().id{
            message_builder::MessageIdTemplate::ExtId(_) |
            message_builder::MessageIdTemplate::StdId(_) => {
                network_set.insert(message, types)
            }
            _ => continue,
        }
    }

    for message in messages {
        match message.0.borrow().id{
            message_builder::MessageIdTemplate::AnyStd(_) |
            message_builder::MessageIdTemplate::AnyExt(_) |
            message_builder::MessageIdTemplate::AnyAny(_) => {
                network_set.insert(message, types);
            }
            _ => continue,
        }
    }

    Ok(())
}
