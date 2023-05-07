use std::collections::HashMap;

use crate::{util::{Broadcastable, NetworkInfo}, broadcast::{BroadcastSender, BroadcastValue}, phase};







pub fn consensus_protocol<T>(initial_value: T, random_generator: fn() -> T, network: NetworkInfo<T>) -> T
where
    T: Broadcastable,
{
    let NetworkInfo {
        id,
        senders,
        mut receiver,
    } = network;
    let process_count = senders.len();
    let faulty_count = if process_count % 3 == 0 {
        process_count / 3 - 1
    } else {
        process_count / 3
    };

    let mut phase_counter = 0;
    let mut early_messages = HashMap::new();
    let sender = BroadcastSender::new(senders);
    let mut current_value = BroadcastValue::new(initial_value, false);

    let mut decided = false;
    while !decided {
        (current_value, early_messages, receiver) = phase::phase(
            id,
            process_count,
            faulty_count,
            phase_counter,
            current_value,
            sender.clone(),
            receiver,
            early_messages,
            random_generator,
        );
        if current_value.decided {
            decided = true;
        }
        phase_counter += 1;
    }
    current_value.value
}
