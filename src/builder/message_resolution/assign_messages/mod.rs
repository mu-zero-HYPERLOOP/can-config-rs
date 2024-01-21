use crate::builder::MessagePriority;

use super::set_assignment::AssignedBus;



pub fn assign_messages(bus : &AssignedBus) {
 
    let bus_name = bus.bus_name();
    let bucket_layout = bus.bucket_layout();

    let mut bucket_offsets = [0usize;MessagePriority::count()];
    let mut bucket_acc = 0;
    for prio in 0..MessagePriority::count() {
        bucket_offsets[prio] = bucket_acc;
        bucket_acc += bucket_layout.bucket_size(prio)
    }

    for set in bus.sets() {
        let setcode = set.setcode();
        let setcode_len = set.setcode_len();
        for prio in 0..MessagePriority::count() {
            for (i, message) in set.messages_with_priority(prio).iter().enumerate() {
                let id = ((bucket_offsets[prio] + i) as u32) << setcode_len | setcode;
                if set.ide() {
                    message.set_ext_id(id);
                }else {
                    message.set_std_id(id);
                }
                message.assign_bus(bus_name);
            }
        }

    }
}
