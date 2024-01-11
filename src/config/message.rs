use std::{fmt::Display, time::Duration, sync::OnceLock};

use super::{ConfigRef, MessageEncoding, SignalRef, Visibility, bus::BusRef, stream::StreamRef, CommandRef};


#[derive(Debug)]
pub enum MessageUsage {
    Stream(StreamRef),
    CommandReq(CommandRef),
    CommandResp(CommandRef),
    GetResp,
    GetReq,
    SetResp,
    SetReq,
    Heartbeat,
    External{interval : Duration},
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum MessageId {
    StandardId(u32),
    ExtendedId(u32),
}

pub type MessageRef = ConfigRef<Message>;

#[derive(Debug)]
pub struct Message {
    name: String,
    description: Option<String>,
    id: MessageId,
    encoding: Option<MessageEncoding>,
    signals: Vec<SignalRef>,
    visibility: Visibility,
    dlc : u8,
    bus : BusRef,
    usage : OnceLock<MessageUsage>,
}


impl Message {
    pub fn new(name : String,
               description : Option<String>,
               id : MessageId,
               encoding : Option<MessageEncoding>,
               signals : Vec<SignalRef>,
               visibility : Visibility, dlc : u8,
               bus : BusRef) -> Self {
        Self {
            name,
            description,
            id,
            encoding,
            signals,
            visibility,
            dlc,
            bus,
            usage : OnceLock::new(),
        }
    }
    pub fn usage(&self) -> &MessageUsage {
        self.usage.get().expect("Karl fucked up big time (message usage was not set property while building!)")
    }
    pub fn __set_usage(&self, usage : MessageUsage) {
        self.usage.set(usage).expect("__set_usage can only be called once (when calling NetworkBuilder::build(&self))");
    }
    pub fn __get_usage(&self) -> &OnceLock<MessageUsage> {
        &self.usage
    }

    pub fn id(&self) -> &MessageId {
        &self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&str> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn encoding(&self) -> Option<&MessageEncoding> {
        self.encoding.as_ref()
    }
    pub fn signals(&self) -> &Vec<SignalRef> {
        &self.signals
    }
    pub fn dlc(&self) -> u8 { 
        self.dlc
    }
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }
    pub fn bus(&self) -> &BusRef {
        &self.bus
    }
}


impl Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            MessageId::StandardId(id) => write!(f, "0x{:X} ({id})", id),
            MessageId::ExtendedId(id) => write!(f, "0x{:X}x ({id})", id),
        }
    }
}
