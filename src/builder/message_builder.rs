use crate::{errors, config::{signal::Signal, Visibility}};

use super::{make_builder_ref, BuilderRef, NetworkBuilder, NodeBuilder};


#[derive(Debug)]
pub enum MessagePriority {
    Default,
    Realtime,
    High,
    Normal,
    Low,
    SuperLow,
}

#[derive(Debug)]
pub enum MessageIdTemplate {
    StdId(u32),
    ExtId(u32),
    AnyStd(MessagePriority),
    AnyExt(MessagePriority),
    AnyAny(MessagePriority),
}

#[derive(Clone, Debug)]
pub struct MessageBuilder(pub BuilderRef<MessageData>);

#[derive(Debug)]
pub struct MessageData {
    pub name: String,
    pub description: Option<String>,
    pub id: MessageIdTemplate,
    pub format: MessageFormat,
    pub network_builder: NetworkBuilder,
    pub visibility: Visibility,
}

#[derive(Debug)]
pub enum MessageFormat {
    Signals(MessageSignalFormatBuilder),
    Types(MessageTypeFormatBuilder),
    Empty,
}

#[derive(Clone, Debug)]
pub struct MessageSignalFormatBuilder(pub BuilderRef<MessageSignalFormatData>);
#[derive(Debug)]
pub struct MessageSignalFormatData(pub Vec<Signal>);
#[derive(Clone, Debug)]
pub struct MessageTypeFormatBuilder(pub BuilderRef<MessageTypeFormatData>);
#[derive(Debug)]
pub struct MessageTypeFormatData(pub Vec<(String, String)>);


impl MessagePriority {
    pub fn min_id(&self) -> u32 {
        match &self {
            MessagePriority::Default => 800,
            MessagePriority::Realtime => 0,
            MessagePriority::High => 400,
            MessagePriority::Normal => 800,
            MessagePriority::Low => 1200,
            MessagePriority::SuperLow => 1600,
        }
    }
}

impl MessageBuilder {
    pub fn new(name: &str, network_builder: &NetworkBuilder) -> MessageBuilder {
        MessageBuilder(make_builder_ref(MessageData {
            name: name.to_owned(),
            description: None,
            id: MessageIdTemplate::AnyAny(MessagePriority::Default),
            format: MessageFormat::Empty,
            network_builder: network_builder.clone(),
            visibility: Visibility::Global,
        }))
    }
    pub fn hide(&self) {
        let mut message_data = self.0.borrow_mut();
        message_data.visibility = Visibility::Static;
    }
    pub fn set_std_id(&self, id: u32) {
        let mut message_data = self.0.borrow_mut();
        message_data.id = MessageIdTemplate::StdId(id);
    }
    pub fn set_ext_id(&self, id: u32) {
        let mut message_data = self.0.borrow_mut();
        message_data.id = MessageIdTemplate::ExtId(id);
    }
    pub fn set_any_std_id(&self, priority: MessagePriority) {
        let mut message_data = self.0.borrow_mut();
        message_data.id = MessageIdTemplate::AnyStd(priority);
    }
    pub fn set_any_ext_id(&self, priority: MessagePriority) {
        let mut message_data = self.0.borrow_mut();
        message_data.id = MessageIdTemplate::AnyExt(priority);
    }
    pub fn make_signal_format(&self) -> MessageSignalFormatBuilder {
        let mut message_data = self.0.borrow_mut();
        let signal_format_builder = MessageSignalFormatBuilder::new();
        message_data.format = MessageFormat::Signals(signal_format_builder.clone());
        signal_format_builder
    }
    pub fn make_type_format(&self) -> MessageTypeFormatBuilder {
        let mut message_data = self.0.borrow_mut();
        let type_format_builder = MessageTypeFormatBuilder::new();
        message_data.format = MessageFormat::Types(type_format_builder.clone());
        type_format_builder
    }
    pub fn add_description(&self, name: &str) {
        let mut message_data = self.0.borrow_mut();
        message_data.description = Some(name.to_owned());
    }
    pub fn add_transmitter(&self, name: &str) {
        // check if node with {name} exists.
        let message_data = self.0.borrow();
        let mut node_named: Option<NodeBuilder> = None;
        for node in message_data
            .network_builder
            .0
            .borrow()
            .nodes
            .borrow()
            .iter()
        {
            if node.0.borrow().name == name {
                node_named = Some(node.clone());
            }
        }
        let node = match node_named {
            Some(node) => node,
            None => message_data.network_builder.create_node(name),
        };
        node.add_tx_message(&self);
    }
    pub fn add_receiver(&self, name: &str) {
        // check if node with {name} exists.
        let message_data = self.0.borrow();
        let mut node_named: Option<NodeBuilder> = None;
        for node in message_data
            .network_builder
            .0
            .borrow()
            .nodes
            .borrow()
            .iter()
        {
            if node.0.borrow().name == name {
                node_named = Some(node.clone());
            }
        }
        let node = match node_named {
            Some(node) => node,
            None => message_data.network_builder.create_node(name),
        };
        node.add_rx_message(&self);
    }
}

impl MessageSignalFormatBuilder {
    pub fn new() -> MessageSignalFormatBuilder {
        MessageSignalFormatBuilder(make_builder_ref(MessageSignalFormatData(vec![])))
    }
    pub fn add_signal(&self, signal: Signal) -> errors::Result<()> {
        let mut builder_data = self.0.borrow_mut();
        if builder_data.0.iter().any(|s| s.name() == signal.name()) {
            return Err(errors::ConfigError::DuplicatedSignal(format!(
                "Dupplicated signal name in message: {}",
                signal.name()
            )));
        }
        builder_data.0.push(signal);
        Ok(())
    }
}
impl MessageTypeFormatBuilder {
    pub fn new() -> MessageTypeFormatBuilder {
        MessageTypeFormatBuilder(make_builder_ref(MessageTypeFormatData(vec![])))
    }
    pub fn add_type(&self, type_name: &str, value_name: &str) {
        let mut builder_data = self.0.borrow_mut();
        builder_data
            .0
            .push((type_name.to_owned(), value_name.to_owned()));
    }
}
