use super::{stream_builder::{ReceiveStreamBuilder, StreamBuilder}, ObjectEntryBuilder, MessageBuilder, NetworkBuilder, CommandBuilder, BuilderRef, MessagePriority, make_builder_ref};


#[derive(Debug, Clone)]
pub struct NodeBuilder(pub BuilderRef<NodeData>);
#[derive(Debug)]
pub struct NodeData {
    pub name: String,
    pub description: Option<String>,
    pub commands: Vec<CommandBuilder>,
    pub extern_commands: Vec<CommandBuilder>,
    pub network_builder: NetworkBuilder,
    pub rx_messages: Vec<MessageBuilder>,
    pub tx_messages: Vec<MessageBuilder>,
    pub object_entries: Vec<ObjectEntryBuilder>,
    pub tx_streams: Vec<StreamBuilder>,
    pub rx_streams: Vec<ReceiveStreamBuilder>,
}


impl NodeBuilder {
    pub fn new(name: &str, network_builder: &NetworkBuilder) -> NodeBuilder {

        let node_builder = NodeBuilder(make_builder_ref(NodeData {
            name: name.to_owned(),
            description: None,
            network_builder: network_builder.clone(),
            commands: vec![],
            extern_commands: vec![],
            tx_messages: vec![],
            rx_messages: vec![],
            object_entries: vec![],
            tx_streams: vec![],
            rx_streams: vec![],
        }));
        node_builder.add_rx_message(&network_builder._get_req_message());
        node_builder.add_tx_message(&network_builder._get_resp_message());
        node_builder.add_rx_message(&network_builder._set_req_message());
        node_builder.add_tx_message(&network_builder._set_resp_message());

        node_builder
    }
    pub fn add_description(&self, description: &str) {
        let mut node_data = self.0.borrow_mut();
        node_data.description = Some(description.to_owned());
    }
    pub fn add_tx_message(&self, message_builder: &MessageBuilder) {
        let mut node_data = self.0.borrow_mut();
        node_data.tx_messages.push(message_builder.clone());
    }
    pub fn add_rx_message(&self, message_builder: &MessageBuilder) {
        let mut node_data = self.0.borrow_mut();
        node_data.rx_messages.push(message_builder.clone());
    }
    pub fn create_command(&self, name: &str) -> CommandBuilder {
        let command_builder = CommandBuilder::new(name, &self);
        let mut node_data = self.0.borrow_mut();
        node_data.commands.push(command_builder.clone());
        node_data
            .rx_messages
            .push(command_builder.0.borrow().call_message.clone());
        node_data
            .tx_messages
            .push(command_builder.0.borrow().resp_message.clone());
        command_builder
    }
    pub fn add_extern_command(&self, message_builder: &CommandBuilder) {
        let mut node_data = self.0.borrow_mut();
        node_data.extern_commands.push(message_builder.clone());
        node_data
            .rx_messages
            .push(message_builder.0.borrow().resp_message.clone());
        node_data
            .tx_messages
            .push(message_builder.0.borrow().call_message.clone());
    }
    pub fn create_object_entry(&self, name: &str, ty: &str) -> ObjectEntryBuilder {
        let object_entry_builder = ObjectEntryBuilder::new(name, ty);
        let mut node_data = self.0.borrow_mut();
        node_data.object_entries.push(object_entry_builder.clone());
        object_entry_builder
    }
    pub fn create_stream(&self, name: &str) -> StreamBuilder {
        let stream_builder = StreamBuilder::new(name, self.clone());
        let mut node_data = self.0.borrow_mut();
        node_data.tx_streams.push(stream_builder.clone());
        stream_builder
    }

    pub fn receive_stream(&self, tx_node_name: &str, tx_stream_name: &str) -> ReceiveStreamBuilder {
        let node_data = self.0.borrow();
        if tx_node_name == node_data.name {
            panic!("can't receive local stream");
        }
        let network_builder = &node_data.network_builder;
        let tx_node_opt = network_builder
            .0
            .borrow()
            .nodes
            .borrow()
            .iter()
            .find(|n| n.0.borrow().name == tx_node_name)
            .cloned();
        let tx_node = match tx_node_opt {
            Some(tx_node) => tx_node,
            None => network_builder.create_node(tx_node_name),
        };
        let tx_node_data = tx_node.0.borrow();
        let tx_stream_opt = tx_node_data
            .tx_streams
            .iter()
            .find(|s| s.0.borrow().name == tx_stream_name)
            .cloned();
        let tx_stream = match tx_stream_opt {
            Some(tx_stream) => tx_stream,
            None => tx_node.create_stream(tx_stream_name),
        };
        drop(node_data);

        let tx_stream_data = tx_stream.0.borrow();
        self.add_rx_message(&tx_stream_data.message);
        drop(tx_stream_data);


        let mut node_data = self.0.borrow_mut();
        let rx_stream_builder = ReceiveStreamBuilder::new(tx_stream, self.clone());
        node_data.rx_streams.push(rx_stream_builder.clone());


        rx_stream_builder
    }
}
