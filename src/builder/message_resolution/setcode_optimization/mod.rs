use crate::builder::NodeBuilder;

use super::set_minimization::bucket_layout;
use super::set_minimization::{set_identifier::SetIdentifier, MinimizedBus};
use crate::builder::MessagePriority;
use crate::builder::MessageBuilder;

struct MergeSetIdentifier {
    receivers: Vec<NodeBuilder>,
    ide: bool,
    id_prefix: Option<u32>,
}

impl MergeSetIdentifier {
    pub fn new(set_identifier: &SetIdentifier, prefix_len: usize) -> Self {
        let mut rx = set_identifier.receivers().clone();
        rx.sort_by_key(|rx| rx.0.borrow().name.clone());
        let id_prefix = set_identifier
            .id()
            .map(|id| id & (0xFFFFFFFFu32.overflowing_shr(32 - prefix_len as u32).0));
        MergeSetIdentifier {
            receivers: set_identifier.receivers().clone(),
            ide: set_identifier.ide().expect("not supported"),
            id_prefix,
        }
    }
}

impl PartialEq for MergeSetIdentifier {
    fn eq(&self, other: &Self) -> bool {
        if self.ide != other.ide {
            return false;
        }
        if self.id_prefix != other.id_prefix {
            return false;
        }
        for (a, b) in std::iter::zip(&self.receivers, &other.receivers) {
            let a_name = a.0.borrow().name.clone();
            if b.0.borrow().name != a_name {
                return false;
            }
        }
        return true;
    }
}

pub struct MergedSet {
    id : MergeSetIdentifier,
    messages: [Vec<MessageBuilder>; MessagePriority::count()],
}

pub fn optimize_sets(bus: MinimizedBus) -> MinimizedBus {
    bus
}
