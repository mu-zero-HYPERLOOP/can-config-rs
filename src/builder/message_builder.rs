use std::time::Duration;

use crate::{
    config::{signal::Signal, Visibility},
    errors,
};

use super::{bus::BusBuilder, make_builder_ref, BuilderRef, NetworkBuilder, NodeBuilder, stream_builder::StreamBuilder, CommandBuilder};

#[derive(Debug)]
pub enum MessagePriority {
    Default,
    Realtime,
    High,
    Normal,
    Low,
    SuperLow,
}

// #[derive(Debug, Clone)]
// pub enum MessageBuilderUsage {
//     Stream(StreamBuilder),
//     CommandReq(CommandBuilder),
//     CommandResp(CommandBuilder),
//     GetResp,
//     GetReq,
//     SetResp,
//     SetReq,
//     External{interval : Duration},
// }

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
    pub receivers : Vec<NodeBuilder>,
    pub transmitters : Vec<NodeBuilder>,
    pub visibility: Visibility,
    pub bus: Option<BusBuilder>,
    pub expected_interval : Option<Duration>,
    // pub usage : MessageBuilderUsage,
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
    pub fn new(name: &str, network_builder: &NetworkBuilder, expected_interval : Option<Duration>) -> MessageBuilder {
        MessageBuilder(make_builder_ref(MessageData {
            name: name.to_owned(),
            description: None,
            id: MessageIdTemplate::AnyAny(MessagePriority::Default),
            format: MessageFormat::Empty,
            network_builder: network_builder.clone(),
            visibility: Visibility::Global,
            bus: None,
            receivers : vec![],
            transmitters : vec![],
            expected_interval,
            // usage,
        }))
    }
    pub fn assign_bus(&self, bus_name: &str) -> BusBuilder {
        let mut message_data = self.0.borrow_mut();
        if message_data.bus.is_some() {
            println!("WARNING: reassiged bus of message : {}
                     but messages can only be assigned to one bus,
                     if splitting is required it is done automatically by 
                     the id, filter and load balancing code!", message_data.name);
        }
        let network_data = message_data.network_builder.0.borrow_mut();
        let bus = network_data
            .buses
            .borrow()
            .iter()
            .find(|bus| &bus.0.borrow().name == bus_name)
            .cloned();
        drop(network_data);
        match bus {
            Some(bus) => {
                message_data.bus = Some(bus.clone());
                bus
            }
            None => {
                let bus = message_data.network_builder.create_bus(bus_name);
                message_data.bus = Some(bus.clone());
                bus
            }
        }
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
    pub fn add_transmitter(&self, node_name: &str) {
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
            if node.0.borrow().name == node_name {
                node_named = Some(node.clone());
            }
        }
        let node = match node_named {
            Some(node) => node,
            None => message_data.network_builder.create_node(node_name),
        };
        node.0.borrow_mut().tx_messages.push(self.clone());
        drop(message_data);
        self.0.borrow_mut().transmitters.push(node);
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
        node.0.borrow_mut().rx_messages.push(self.clone());
        drop(message_data);
        self.0.borrow_mut().receivers.push(node);
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
