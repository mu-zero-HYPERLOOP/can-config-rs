
pub mod errors;
pub mod config;
pub mod builder;

#[cfg(test)]
mod tests {
    use crate::{builder::NetworkBuilder, config::{Type, SignalType, signal::Signal}};


    #[test]
    fn set_baudrate() {
        let builder = NetworkBuilder::new();
        builder.set_baudrate(500000);
        let network = builder.build().unwrap();
        assert_eq!(network.baudrate(), 500000);
    }

    #[test]
    fn create_node() {
        let builder = NetworkBuilder::new();
        builder.create_node("foo");
        let network = builder.build().unwrap();
        assert!(network.nodes().iter().any(|n| n.name() == "foo"));
    }

    #[test]
    fn add_node_description() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        node.add_description("bar");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        assert_eq!(node.description(), Some(&String::from("bar")));
    }

    #[test]
    fn create_object_entry_u32() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        node.create_object_entry("bar", "u32");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(oe) = node.object_entries().iter().find(|oe| oe.name() == "bar") else {
            panic!("Object Entry not defined");
        };
        assert_eq!(oe.name(), "bar");
        let ty = oe.ty() as &Type;
        assert_eq!(ty, &Type::Primitive(SignalType::UnsignedInt { size: 32 }));
    }

    #[test]
    fn create_object_entry_i32() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        node.create_object_entry("bar", "i32");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(oe) = node.object_entries().iter().find(|oe| oe.name() == "bar") else {
            panic!("Object Entry not defined");
        };
        assert_eq!(oe.name(), "bar");
        let ty = oe.ty() as &Type;
        assert_eq!(ty, &Type::Primitive(SignalType::SignedInt { size: 32 }));
    }

    #[test]
    fn create_message() {
        let builder = NetworkBuilder::new();
        builder.create_message("bar");
        let network = builder.build().unwrap();
        assert!(network.messages().iter().any(|m| m.name() == "bar"));
    }

    #[test]
    fn create_message_signal_format() {
        let builder = NetworkBuilder::new();
        let message = builder.create_message("bar");
        let format = message.make_signal_format();
        format
            .add_signal(Signal::new(
                "foo1",
                None,
                SignalType::UnsignedInt { size: 32 },
            ))
            .unwrap();
        format
            .add_signal(Signal::new(
                "foo2",
                None,
                SignalType::SignedInt { size: 32 },
            ))
            .unwrap();
        let network = builder.build().unwrap();
        let Some(message) = network.messages().iter().find(|m| m.name() == "bar") else {
            panic!("Node not defined");
        };
        assert_eq!(message.signals().len(), 2);
        assert_eq!(message.signals()[0].byte_offset(), 0);
        assert_eq!(
            message.signals()[0].ty(),
            &SignalType::UnsignedInt { size: 32 }
        );
        assert_eq!(message.signals()[0].size(), 32);
        assert_eq!(message.signals()[1].byte_offset(), 32);
        assert_eq!(
            message.signals()[1].ty(),
            &SignalType::SignedInt { size: 32 }
        );
        assert_eq!(message.signals()[1].size(), 32);
    }

    #[test]
    fn create_message_type_format() {
        let builder = NetworkBuilder::new();
        let message = builder.create_message("bar");
        let format = message.make_type_format();
        format.add_type("u32", "foo1");
        format.add_type("i32", "foo2");
        let network = builder.build().unwrap();
        let Some(message) = network.messages().iter().find(|m| m.name() == "bar") else {
            panic!("Node not defined");
        };
        assert_eq!(message.signals().len(), 2);
        assert_eq!(message.signals()[0].byte_offset(), 0);
        assert_eq!(
            message.signals()[0].ty(),
            &SignalType::UnsignedInt { size: 32 }
        );
        assert_eq!(message.signals()[0].size(), 32);
        assert_eq!(message.signals()[1].byte_offset(), 32);
        assert_eq!(
            message.signals()[1].ty(),
            &SignalType::SignedInt { size: 32 }
        );
        assert_eq!(message.signals()[1].size(), 32);

        let Some(encoding) = message.encoding() else {
            panic!();
        };
        assert_eq!(encoding.len(), 2);
        //assert_eq!(encoding[0].ty, Type::Primitive
        assert_eq!(
            encoding[0].ty() as &Type,
            &Type::Primitive(SignalType::UnsignedInt { size: 32 })
        );
        assert_eq!(
            encoding[1].ty() as &Type,
            &Type::Primitive(SignalType::SignedInt { size: 32 })
        );
    }

    #[test]
    fn create_message_transmitter_implicit() {
        let builder = NetworkBuilder::new();
        builder.create_node("foo");
        let message = builder.create_message("bar");
        message.add_transmitter("foo");
        let network = builder.build().unwrap();
        let Some(message) = network.messages().iter().find(|m| m.name() == "bar") else {
            panic!("Message not defined");
        };
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == message.name()));
    }

    #[test]
    fn create_message_transmitter_explicit() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        let message = builder.create_message("bar");
        node.add_tx_message(&message);
        let network = builder.build().unwrap();
        let Some(message) = network.messages().iter().find(|m| m.name() == "bar") else {
            panic!("Message not defined");
        };
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == message.name()));
    }

    #[test]
    fn create_message_receiver_implicit() {
        let builder = NetworkBuilder::new();
        builder.create_node("foo");
        let message = builder.create_message("bar");
        message.add_receiver("foo");
        let network = builder.build().unwrap();
        let Some(message) = network.messages().iter().find(|m| m.name() == "bar") else {
            panic!("Message not defined");
        };
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        assert!(node
            .rx_messages()
            .iter()
            .any(|m| m.name() == message.name()));
    }

    #[test]
    fn create_message_receiver_explicit() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        let message = builder.create_message("bar");
        node.add_rx_message(&message);
        let network = builder.build().unwrap();
        let Some(message) = network.messages().iter().find(|m| m.name() == "bar") else {
            panic!("Message not defined");
        };
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        assert!(node
            .rx_messages()
            .iter()
            .any(|m| m.name() == message.name()));
    }

    #[test]
    fn create_command() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        node.create_command("bar");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(command) = node.commands().iter().find(|c| c.name() == "bar") else {
            panic!("Message not defined");
        };
        assert!(node
            .rx_messages()
            .iter()
            .any(|m| m.name() == command.tx_message().name()));
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == command.rx_message().name()));
    }

    #[test]
    fn create_command_argument() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        let command = node.create_command("bar");
        command.add_argument("x", "u31");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(command) = node.commands().iter().find(|c| c.name() == "bar") else {
            panic!("Message not defined");
        };
        assert!(node
            .rx_messages()
            .iter()
            .any(|m| m.name() == command.tx_message().name()));
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == command.rx_message().name()));

        let req = command.tx_message();
        let Some(encoding) = req.encoding() else {
            panic!("Message type format not defined");
        };
        assert!(encoding
            .iter()
            .any(|e| e.ty() as &Type == &Type::Primitive(SignalType::UnsignedInt { size: 31 }),));
    }

    #[test]
    fn create_command_callee_implicit() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        let command = node.create_command("bar");
        command.add_callee("callee");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(callee) = network.nodes().iter().find(|n| n.name() == "callee") else {
            panic!("Node not defined");
        };
        let Some(command) = node.commands().iter().find(|c| c.name() == "bar") else {
            panic!("Message not defined");
        };
        assert!(node
            .rx_messages()
            .iter()
            .any(|m| m.name() == command.tx_message().name()));
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == command.rx_message().name()));

        assert!(callee
            .tx_messages()
            .iter()
            .any(|m| m.name() == command.tx_message().name()));
        assert!(callee
            .rx_messages()
            .iter()
            .any(|m| m.name() == command.rx_message().name()));
        assert!(callee
            .extern_commands()
            .iter()
            .any(|(node_name, extern_command)| {
                if node_name != "foo" {
                    return false;
                }
                extern_command.name() == command.name()
            }));
    }

    #[test]
    fn create_command_callee_explicit() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        let command = node.create_command("bar");
        let callee = builder.create_node("callee");
        callee.add_extern_command(&command);
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(callee) = network.nodes().iter().find(|n| n.name() == "callee") else {
            panic!("Node not defined");
        };
        let Some(command) = node.commands().iter().find(|c| c.name() == "bar") else {
            panic!("Message not defined");
        };
        assert!(node
            .rx_messages()
            .iter()
            .any(|m| m.name() == command.tx_message().name()));
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == command.rx_message().name()));

        assert!(callee
            .tx_messages()
            .iter()
            .any(|m| m.name() == command.tx_message().name()));
        assert!(callee
            .rx_messages()
            .iter()
            .any(|m| m.name() == command.rx_message().name()));
        assert!(callee
            .extern_commands()
            .iter()
            .any(|(node_name, extern_command)| {
                if node_name != "foo" {
                    return false;
                }
                extern_command.name() == command.name()
            }));
    }

    #[test]
    fn create_stream() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        node.create_stream("realtime");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(stream) = node.tx_streams().iter().find(|c| c.name() == "realtime") else {
            panic!("Stream not defined");
        };
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == stream.message().name()));
        assert!(node.tx_streams().iter().any(|s| s.name() == "realtime"));
    }

    #[test]
    fn create_stream_entry() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        node.create_object_entry("something", "u32");
        let stream = node.create_stream("realtime");
        stream.add_entry("something");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(stream) = node.tx_streams().iter().find(|c| c.name() == "realtime") else {
            panic!("Stream not defined");
        };
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == stream.message().name()));
        assert!(node.tx_streams().iter().any(|s| s.name() == "realtime"));
        assert_eq!(stream.message().signals().len(), 1);

    }

    #[test]
    fn create_stream_receiver() {
        let builder = NetworkBuilder::new();
        let node = builder.create_node("foo");
        node.create_stream("realtime");
        let receiver = builder.create_node("receiver");
        receiver.receive_stream("foo", "realtime");
        let network = builder.build().unwrap();
        let Some(node) = network.nodes().iter().find(|n| n.name() == "foo") else {
            panic!("Node not defined");
        };
        let Some(stream) = node.tx_streams().iter().find(|c| c.name() == "realtime") else {
            panic!("Stream not defined");
        };
        assert!(node
            .tx_messages()
            .iter()
            .any(|m| m.name() == stream.message().name()));
        assert!(node.tx_streams().iter().any(|s| s.name() == "realtime"));

        let Some(receiver) = network.nodes().iter().find(|n| n.name() == "receiver") else {
            panic!("Node not defined");
        };
        assert!(receiver.rx_streams().iter().any(|s| s.name() == "realtime"));
        assert!(receiver.rx_messages().iter().any(|m| m.name() == stream.message().name()));
        
    }

    // this is not a clean test it just checks that something is not crashing
    #[test]
    fn create_stream_receiver_2_reproduce() {
        let builder = NetworkBuilder::new();
        let secu = builder.create_node("secu");
        secu.create_object_entry("cpu_temperature", "d8<-10..100>");
        secu.create_object_entry("bcu_temperature", "d8<-10..100>");

        let tx_stream = secu.create_stream("ecu_temperatures");
        tx_stream.add_entry("cpu_temperature");
        tx_stream.add_entry("bcu_temperature");
        
        let master = builder.create_node("master");
        let rx_stream = master.receive_stream("secu", "ecu_temperatures");
        rx_stream.map("cpu_temperature", "secu_cpu_temperature");

        builder.build().unwrap();
    }
}
