use super::{ConfigRef, ObjectEntryRef, MessageRef, Visibility};


pub type StreamRef = ConfigRef<Stream>;

#[derive(Debug)]
pub struct Stream {
    name: String,
    description: Option<String>,
    mappings: Vec<Option<ObjectEntryRef>>,
    message: MessageRef,
    visibility: Visibility,
}

impl Stream {
    pub fn new(name : String, description : Option<String>,
               mappings : Vec<Option<ObjectEntryRef>>,
               message : MessageRef,
               visibility : Visibility) -> Self {
        Self {
            name,
            description,
            mappings,
            message,
            visibility
        }
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
    pub fn mapping(&self) -> &Vec<Option<ObjectEntryRef>> {
        &self.mappings
    }
    pub fn message(&self) -> &MessageRef {
        &self.message
    }
}
