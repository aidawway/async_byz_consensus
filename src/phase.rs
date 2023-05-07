use std::collections::HashMap;

use crossbeam::channel::Receiver;

use crate::{
    broadcast::{BroadcastSender, BroadcastValue},
    messaging::Message,
    round, selection_protocol,
    util::{self, Broadcastable},
    validation::ValidatedMessageSet,
};

pub fn phase<T>(
    id: usize,
    process_count: usize,
    faulty_count: usize,
    phase_counter: usize,
    initial_value: BroadcastValue<T>,
    sender: BroadcastSender<T>,
    receiver: Receiver<Message<T>>,
    early_messages: HashMap<usize, Vec<Message<T>>>,
    random_value: fn() -> T,
) -> (
    BroadcastValue<T>,
    HashMap<usize, Vec<Message<T>>>,
    Receiver<Message<T>>,
)
where
    T: Broadcastable,
{
    let mut round_counter = 3 * phase_counter;
    let mut current_value = initial_value;
    //TODO avoid hardcoding
    let (validated_messages, early_messages, receiver) = round::round(
        round_counter,
        id,
        current_value,
        process_count,
        early_messages,
        ValidatedMessageSet::new(),
        receiver,
        sender.clone(),
    );
    current_value =
        selection_protocol::selection_protocol(round_counter, process_count, &validated_messages)
            .expect("Expected phase stage one to have a majority");
    round_counter += 1;

    let (validated_messages, early_messages, receiver) = round::round(
        round_counter,
        id,
        current_value.clone(),
        process_count,
        early_messages,
        validated_messages,
        receiver,
        sender.clone(),
    );
    current_value =
        selection_protocol::selection_protocol(round_counter, process_count, &validated_messages)
            .unwrap_or(current_value);
    round_counter += 1;

    let (validated_messages, early_messages, receiver) = round::round(
        round_counter,
        id,
        current_value,
        process_count,
        early_messages,
        validated_messages,
        receiver,
        sender.clone(),
    );
    let final_value =
        selection_protocol::selection_protocol(round_counter, process_count, &validated_messages)
            .unwrap_or(BroadcastValue::new(random_value(), false));

    (final_value, early_messages, receiver)
}
