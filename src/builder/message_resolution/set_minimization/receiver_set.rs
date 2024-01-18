use super::bucket_layout::BucketLayoutCommit;
use super::MinimizedSet;
use super::{
    bucket_layout::BucketLayout, priority_bucket::PriorityBucket, set_identifier::SetIdentifier,
};
use crate::builder::{MessageBuilder, MessagePriority};

pub struct ReceiverSet {
    id: SetIdentifier,
    priority_buckets: [PriorityBucket; MessagePriority::count()],
}

impl ReceiverSet {
    pub fn new(id: SetIdentifier) -> Self {
        Self {
            id,
            priority_buckets: std::array::from_fn(|_| PriorityBucket::new()),
        }
    }
    pub fn identifier(&self) -> &SetIdentifier {
        &self.id
    }
    pub fn insert_message(&mut self, message: &MessageBuilder) {
        match message.0.borrow().id {
            crate::builder::message_builder::MessageIdTemplate::StdId(id)
            | crate::builder::message_builder::MessageIdTemplate::ExtId(id) => {
                panic!("fixed ids are not supported by set_minimization")
            }
            crate::builder::message_builder::MessageIdTemplate::AnyStd(prio) |
            crate::builder::message_builder::MessageIdTemplate::AnyExt(prio) |
            crate::builder::message_builder::MessageIdTemplate::AnyAny(prio) => {
                self.priority_buckets[prio.to_u32() as usize].insert_message(message);
            }
        }
    }
    pub fn set_count(&self, bucket_layout: &BucketLayout) -> usize {
        self.priority_buckets
            .iter()
            .enumerate()
            .map(|(p, b)| b.required_sets(bucket_layout.bucket_size(p)))
            .max()
            .expect("priority_buckets should not be empty!")
    }

    pub fn priorioty_bucket(&self, priority : usize) -> &PriorityBucket {
        &self.priority_buckets[priority]
    }

    pub fn min_commit_to_merge(&self, bucket_layout: &BucketLayout) -> Option<BucketLayoutCommit> {
        let mut inc = [0usize;MessagePriority::count()];
        for prio in 0..MessagePriority::count() {
            inc[prio] = self.priority_buckets[prio].required_inc_for_merge(bucket_layout.bucket_size(prio)).unwrap_or(0);
        }
        if inc.iter().sum::<usize>() == 0 {
            None
        }else {
            Some(BucketLayoutCommit::new(inc))
        }
    }

    pub fn to_sets(&self, bucket_layout: &BucketLayout) -> Vec<MinimizedSet> {
        let mut min_sets_priority_buckets: Vec<[Vec<MessageBuilder>; MessagePriority::count()]> =
            vec![];
        for _ in 0..self.set_count(bucket_layout) {
            min_sets_priority_buckets.push(std::array::from_fn(|_| vec![]));
        }

        for priority in 0..MessagePriority::count() {
            let bucket_messages = self.priority_buckets[priority].messages();
            let mut insert_set_id = 0;
            for bucket_message in bucket_messages {
                let min_set_priority_bucket = &mut min_sets_priority_buckets[insert_set_id][priority];
                min_set_priority_bucket.push(bucket_message.clone());
                if min_set_priority_bucket.len() == bucket_layout.bucket_size(priority) {
                    insert_set_id += 1;
                }
            }
        }

        let minimized_sets : Vec<MinimizedSet> = min_sets_priority_buckets
            .into_iter()
            .map(|min_set| MinimizedSet::new(min_set))
            .collect();
        assert_eq!(minimized_sets.len(), self.set_count(bucket_layout));

        minimized_sets
    }
}
