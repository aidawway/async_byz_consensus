use std::collections::{BTreeMap, HashMap};

use crate::{
    broadcast::BroadcastValue,
    util::{self, Broadcastable},
};

pub struct ValidatedMessageSet<T> {
    messages: HashMap<T, usize>,
}

impl<T> ValidatedMessageSet<T>
where
    T: Broadcastable,
{
    pub fn new() -> ValidatedMessageSet<T> {
        ValidatedMessageSet {
            messages: HashMap::new(),
        }
    }

    pub fn add(&mut self, value: BroadcastValue<T>) {
        *self.messages.entry(value.value).or_insert(0) += 1;
    }

    //TODO connect with selection_protocol
    pub fn validate(&self, round: usize, process_count: usize, value: &BroadcastValue<T>) -> bool {
        if self.messages.len() == 0 {
            return true;
        }

        let faulty_count = util::faulty_count(process_count);
        match round % 3 {
            0 => self.validate_threshold(value, (process_count - faulty_count) / 2),
            1 => self.validate_threshold(value, process_count / 2),
            2 => {
                if value.decided {
                    self.validate_threshold(value, 2 * faulty_count)
                } else {
                    self.validate_threshold(value, faulty_count)
                }
            }
            _ => unreachable!(),
        }
    }

    fn validate_threshold(&self, value: &BroadcastValue<T>, threshold: usize) -> bool {
        self.messages.contains_key(&value.value)
            && *self.messages.get(&value.value).unwrap() > threshold
    }

    pub fn get_threshold_majority(&self, threshold: usize, include_undecided: bool) -> Option<T> {
        self.messages
            .iter()
            .fold(
                (None, threshold), //If no elements have at least threshold messages then return None
                |(current_value, threshold), (new_value, count)| {
                    // Keep the element with the larger count
                    if *count > threshold {
                        (Some(new_value.clone()), *count)
                    } else {
                        (current_value, threshold)
                    }
                },
            )
            .0
            .clone()
    }
}
