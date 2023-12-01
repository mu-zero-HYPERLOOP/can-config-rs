use can_config_rs::config::types;

extern crate can_config_rs;


fn main() {
    let network_builder = can_config_rs::builder::NetworkBuilder::new();
    network_builder.create_node("secu");

    let network_config = network_builder.build().unwrap();
    let secu = network_config.nodes().iter().find(|n| n.name() == "secu").unwrap();
    let messages = secu.tx_messages();
    
    let get_resp_message = messages.iter().find(|m| m.name() == "get_resp").unwrap();
    println!("dlc = {}", get_resp_message.dlc());
    println!("signals = {:?}", get_resp_message.signals());

}
