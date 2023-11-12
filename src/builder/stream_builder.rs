use crate::config::Visibility;

use super::{NodeBuilder, make_builder_ref, BuilderRef, MessageBuilder, MessageTypeFormatBuilder, ObjectEntryBuilder, MessagePriority};


#[derive(Debug, Clone)]
pub struct StreamBuilder(pub BuilderRef<StreamData>);
#[derive(Debug)]
pub struct StreamData {
    pub name: String,
    pub description: Option<String>,
    pub message: MessageBuilder,
    pub format: MessageTypeFormatBuilder,
    pub tx_node: NodeBuilder,
    pub object_entries: Vec<ObjectEntryBuilder>,
    pub visbility: Visibility,
}

#[derive(Debug, Clone)]
pub struct ReceiveStreamBuilder(pub BuilderRef<ReceiveStreamData>);
#[derive(Debug)]
pub struct ReceiveStreamData {
    pub stream_builder: StreamBuilder,
    pub rx_node: NodeBuilder,
    pub object_entries: Vec<(usize, ObjectEntryBuilder)>,
    pub visibility: Visibility,
}

impl StreamBuilder {
    pub fn new(name: &str, node_builder: NodeBuilder) -> StreamBuilder {
        let node_data = node_builder.0.borrow();
        let message = node_data
            .network_builder
            .create_message(&format!("{}_stream_{name}", node_builder.0.borrow().name));
        drop(node_data);
        node_builder.add_tx_message(&message);
        message.hide();
        message.set_any_std_id(MessagePriority::Normal);
        let format = message.make_type_format();

        StreamBuilder(make_builder_ref(StreamData {
            name: name.to_owned(),
            description: None,
            message,
            format,
            tx_node: node_builder,
            object_entries: vec![],
            visbility: Visibility::Global,
        }))
    }
    pub fn hide(&self) {
        let mut stream_data = self.0.borrow_mut();
        stream_data.visbility = Visibility::Static;
    }
    pub fn add_description(&self, description: &str) {
        let mut stream_data = self.0.borrow_mut();
        stream_data.description = Some(description.to_owned());
    }
    pub fn add_entry(&self, name: &str) {
        let mut stream_data = self.0.borrow_mut();
        let node = stream_data.tx_node.clone();
        let node_data = node.0.borrow();
        let oe = node_data
            .object_entries
            .iter()
            .find(|oe| oe.0.borrow().name == name)
            .cloned()
            .unwrap_or_else(|| node.create_object_entry(name, "u1"));
        stream_data.object_entries.push(oe.clone());
        let oe_data = oe.0.borrow();
        stream_data.format.add_type(&oe_data.ty, &oe_data.name);
    }
}

impl ReceiveStreamBuilder {
    pub fn new(stream_builder: StreamBuilder, rx_node: NodeBuilder) -> ReceiveStreamBuilder {
        ReceiveStreamBuilder(make_builder_ref(ReceiveStreamData {
            stream_builder,
            rx_node,
            object_entries: vec![],
            visibility: Visibility::Global,
        }))
    }
    pub fn hide(&self) {
        let mut rx_stream_data = self.0.borrow_mut();
        rx_stream_data.visibility = Visibility::Static;
    }
    pub fn map(&self, from: &str, to: &str) {
        // resolve from
        let mut rx_stream_data = self.0.borrow_mut();
        let tx_stream_builder = rx_stream_data.stream_builder.clone();
        let tx_stream_data = tx_stream_builder.0.borrow();
        let opt_pos = tx_stream_data
            .object_entries
            .iter()
            .position(|oe| oe.0.borrow().name == from);
        let Some(pos) = opt_pos else {
            panic!("invalid rx stream mapping");
        };
        // resolve to
        let oe_opt = rx_stream_data
            .rx_node
            .0
            .borrow()
            .object_entries
            .iter()
            .find(|oe| oe.0.borrow().name == to)
            .cloned();
        let oe = match oe_opt {
            Some(oe) => {
                assert_eq!(
                    oe.0.borrow().ty,
                    tx_stream_data.object_entries[pos].0.borrow().ty
                );
                oe
            }
            None => {
                let tx_oe = tx_stream_data.object_entries[pos].0.borrow();
                rx_stream_data.rx_node.create_object_entry(to, &tx_oe.ty)
            }
        };
        rx_stream_data.object_entries.push((pos, oe));
    }
}
