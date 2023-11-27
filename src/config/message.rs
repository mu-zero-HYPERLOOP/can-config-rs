use std::fmt::Display;

use super::{ConfigRef, MessageEncoding, SignalRef, Visibility};


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
}


impl Message {
    pub fn new(name : String,
               description : Option<String>,
               id : MessageId,
               encoding : Option<MessageEncoding>,
               signals : Vec<SignalRef>,
               visibility : Visibility, dlc : u8) -> Self {
        Self {
            name,
            description,
            id,
            encoding,
            signals,
            visibility,
            dlc,
        }
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
}


impl Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            MessageId::StandardId(id) => write!(f, "0x{:X} ({id})", id),
            MessageId::ExtendedId(id) => write!(f, "0x{:X}x ({id})", id),
        }
    }
}
