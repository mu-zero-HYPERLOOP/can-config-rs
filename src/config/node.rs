use super::{ConfigRef, TypeRef, CommandRef, stream::StreamRef, MessageRef, ObjectEntryRef, Message};


pub type NodeRef = ConfigRef<Node>;


#[derive(Debug)]
pub struct Node {
    name: String,
    description: Option<String>,

    types: Vec<TypeRef>,

    commands: Vec<CommandRef>,
    extern_commands: Vec<(String, CommandRef)>,

    tx_streams: Vec<StreamRef>,
    rx_streams: Vec<StreamRef>,

    rx_messages: Vec<MessageRef>,
    tx_messages: Vec<MessageRef>,

    object_entries: Vec<ObjectEntryRef>,
    get_resp_message: MessageRef,
    get_req_message: MessageRef,
    set_resp_message: MessageRef,
    set_req_message: MessageRef,
}

impl Node {
    pub fn new(name : String, description : Option<String>,
               types : Vec<TypeRef>,
               commands : Vec<CommandRef>,
               extern_commands : Vec<(String, CommandRef)>,
               tx_streams : Vec<StreamRef>,
               rx_streams : Vec<StreamRef>,
               rx_messages : Vec<MessageRef>,
               tx_messages : Vec<MessageRef>,
               object_entries : Vec<ObjectEntryRef>,
               get_resp_message : MessageRef,
               get_req_message  : MessageRef,
               set_resp_message : MessageRef,
               set_req_message : MessageRef)-> Self{
        Self {
            name,
            description,
            types,
            commands,
            extern_commands,
            tx_streams,
            rx_streams,
            rx_messages,
            tx_messages,
            object_entries,
            get_resp_message,
            get_req_message,
            set_resp_message,
            set_req_message
        }
    }

               
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn types(&self) -> &Vec<TypeRef> {
        &self.types
    }
    pub fn commands(&self) -> &Vec<CommandRef> {
        &self.commands
    }
    pub fn extern_commands(&self) -> &Vec<(String, CommandRef)> {
        &self.extern_commands
    }
    pub fn extern_commands_mut(&mut self) -> &mut Vec<(String, CommandRef)> {
        &mut self.extern_commands
    }
    pub fn tx_streams(&self) -> &Vec<StreamRef> {
        &self.tx_streams
    }
    pub fn rx_streams(&self) -> &Vec<StreamRef> {
        &self.rx_streams
    }
    pub fn rx_streams_mut(&mut self) -> &mut Vec<StreamRef> {
        &mut self.rx_streams
    }
    pub fn tx_messages(&self) -> &Vec<MessageRef> {
        &self.tx_messages
    }
    pub fn rx_messages(&self) -> &Vec<MessageRef> {
        &self.rx_messages
    }
    pub fn object_entries(&self) -> &Vec<ObjectEntryRef> {
        &self.object_entries
    }
    pub fn description(&self) -> Option<&String> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn get_resp_message(&self) -> &Message {
        &self.get_resp_message
    }
    pub fn get_req_message(&self) -> &Message {
        &self.get_req_message
    }
    pub fn set_resp_message(&self) -> &Message {
        &self.set_resp_message
    }
    pub fn set_req_message(&self) -> &Message {
        &self.set_req_message
    }
}
