use crate::config::{NetworkBuilder};

mod config;
mod errors;

fn main() {
    let builder = NetworkBuilder::new();
    builder.set_baudrate(10000000);


    let message = builder.create_message("OpticalSpeed_Data");
    message.set_std_id(100);
    message.add_receiver("secu");

    let secu = builder.create_node("secu");
    secu.create_object_entry("cpu_temperature", "XYZ");

    let realtime = secu.create_stream("realtime");
    realtime.add_entry("cpu_temperature");

    let command = secu.create_command("configure_filters");

    command.add_argument("x", "u32");

    let master = builder.create_node("master");
    let stream_receiver = master.receive_stream("secu", "realtime");
    stream_receiver.map("cpu_temperature", "secu_temp");

    master.add_extern_command(&command);



    let xyz = builder.define_struct("XYZ");
    xyz.add_attribute("x", "u32").unwrap();
    xyz.add_attribute("y", "u32").unwrap();
    xyz.add_attribute("z", "u32").unwrap();

    let network = builder.build().unwrap();

    println!("{network}");
}

