use std::collections::{BTreeMap, HashMap};

use crate::{broadcast::BroadcastValue, util::Broadcastable};

pub struct MessageTally {
    decided: usize,
    undecided: usize,
}

impl MessageTally {
    fn new() -> MessageTally {
        MessageTally { decided: 0, undecided: 0 }
    }

    pub fn increment_decided(&mut self) {
        self.decided += 1;
    }

    pub fn increment_undecided(&mut self) {
        self.undecided += 1;
    }
}

pub struct ValidatedMessageSet<T> {
    messages: HashMap<T, MessageTally>,
}

impl<T> ValidatedMessageSet<T> 
where T: Broadcastable
{

    pub fn new() -> ValidatedMessageSet<T> {
        ValidatedMessageSet { messages: HashMap::new() }
    }

    pub fn add(&mut self, value: BroadcastValue<T>) {
        let entry = self.messages.entry(value.value).or_insert(MessageTally::new());
        if value.decided {
            entry.increment_decided();
        } else {
            entry.increment_undecided();
        }
    }

    pub fn get_threshold_majority(
        &self,
        threshold: usize,
        include_undecided: bool
        ) -> Option<T> {
        self.messages
            .iter()
            .fold(
                (None, threshold), //If no elements have at least threshold messages then return None
                |(current_value, threshold), (new_value, MessageTally { decided, undecided })| {
                    let count = if include_undecided {
                        //Include total count
                        decided + undecided
                    } else {
                        //Use only the count of decided messages
                        *decided
                    };

                    // Keep the element with the larger count
                    if count > threshold {
                        (Some(new_value.clone()), count)
                    } else {
                        (current_value, threshold)
                    }
                },
            )
            .0.clone()
    }
}
