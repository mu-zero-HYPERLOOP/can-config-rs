
extern crate can_config_rs;

fn main() {
    let network_builder = can_config_rs::builder::NetworkBuilder::new();
    let bus = network_builder.create_bus("100");
    bus.baudrate(1000000);
    network_builder.create_node("secu");

    let network_config = network_builder.build().unwrap();
    let secu = network_config.nodes().iter().find(|n| n.name() == "secu").unwrap();
    let messages = secu.tx_messages();
    
    let get_resp_message = messages.iter().find(|m| m.name() == "get_resp").unwrap();
    println!("dlc = {}", get_resp_message.dlc());
    println!("signals = {:?}", get_resp_message.signals());

}
