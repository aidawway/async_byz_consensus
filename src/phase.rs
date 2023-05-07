use std::collections::HashMap;

use crossbeam::channel::Receiver;

use crate::{
    broadcast::{BroadcastSender, BroadcastValue},
    messaging::Message,
    round,
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
    let round_counter = phase_counter * 3;
    let (results, early_messages, receiver) = round::round(
        round_counter,
        id,
        initial_value,
        process_count,
        early_messages,
        receiver,
        sender.clone(),
    );
    let mut current_value = stage_one_selection_protocol(process_count, &results)
        .expect("Expected phase stage one to have a majority");

    let (results, early_messages, receiver) = round::round(
        round_counter + 1,
        id,
        current_value.clone(),
        process_count,
        early_messages,
        receiver,
        sender.clone(),
    );
    current_value = stage_two_selection_protocol(process_count, &results).unwrap_or(current_value);

    let (results, early_messages, receiver) = round::round(
        round_counter + 2,
        id,
        current_value.clone(),
        process_count,
        early_messages,
        receiver,
        sender,
    );
    let final_value = stage_three_selection_protocol(process_count, &results)
        .unwrap_or(BroadcastValue::new(random_value(), false));
    (final_value, early_messages, receiver)
}

fn stage_one_selection_protocol<T>(
    _process_count: usize,
    validated: &ValidatedMessageSet<T>,
) -> Option<BroadcastValue<T>>
where
    T: Broadcastable,
{
    //normal majority suffices
    if let Some(value) = validated.get_threshold_majority(0, true) {
        Some(BroadcastValue::new(value, false))
    } else {
        None
    }
}

fn stage_two_selection_protocol<T>(
    process_count: usize,
    validated: &ValidatedMessageSet<T>,
) -> Option<BroadcastValue<T>>
where
    T: Broadcastable,
{
    // Note that process_count >= validated's size
    if let Some(value) = validated.get_threshold_majority(process_count / 2, true) {
        Some(BroadcastValue::new(value, true))
    } else {
        None
    }
}

fn stage_three_selection_protocol<T>(
    process_count: usize,
    validated: &ValidatedMessageSet<T>,
) -> Option<BroadcastValue<T>>
where
    T: Broadcastable,
{
    if let Some(value) =
        validated.get_threshold_majority(util::faulty_count(process_count) * 2, false)
    {
        Some(BroadcastValue::new(value, true))
    } else if let Some(value) =
        validated.get_threshold_majority(util::faulty_count(process_count), false)
    {
        Some(BroadcastValue::new(value, false))
    } else {
        None
    }
}
