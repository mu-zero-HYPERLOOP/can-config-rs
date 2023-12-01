use can_config_rs::config::types;

extern crate can_config_rs;


fn main() {
    let network_builder = can_config_rs::builder::NetworkBuilder::new();
    network_builder.create_node("secu");

    let network_config = network_builder.build().unwrap();
    let secu = network_config.nodes().iter().find(|n| n.name() == "secu").unwrap();
    let types = secu.types();
    
    for ty in types {
        println!("{}", ty.name());
    }

}
