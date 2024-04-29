use std::time::Duration;

use super::{ConfigRef, MessageRef, Visibility, Message};


pub type CommandRef = ConfigRef<Command>;

#[derive(Debug, Hash)]
pub struct Command {
    name: String,
    description: Option<String>,
    tx_message: MessageRef,
    rx_message: MessageRef,
    visibility: Visibility,
    expected_interval : Duration,
}

impl Command {
    pub fn new(name : String,
               description : Option<String>,
               tx_message : MessageRef,
               rx_message : MessageRef,
               visibility : Visibility, 
               expected_interval : Duration) -> Self {
        Self{
            name,
            description,
            tx_message,
            rx_message,
            visibility,
            expected_interval
        }
    }
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }
    pub fn expected_interval(&self) -> &Duration {
        &self.expected_interval
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn description(&self) -> Option<&String> {
        match &self.description {
            Some(some) => Some(&some),
            None => None,
        }
    }
    pub fn tx_message(&self) -> &Message {
        &self.tx_message
    }
    pub fn rx_message(&self) -> &Message {
        &self.rx_message
    }
}
