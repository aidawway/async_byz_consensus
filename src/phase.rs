use std::collections::HashMap;

use crossbeam::channel::Receiver;

use crate::{
    broadcast::{BroadcastSender, BroadcastValue},
    messaging::Message,
    round,
    util::Broadcastable,
};

pub fn phase<T>(
    id: usize,
    process_count: usize,
    faulty_count: usize,
    phase_counter: usize,
    mut current_value: BroadcastValue<T>,
    sender: BroadcastSender<T>,
    receiver: Receiver<Message<T>>,
    early_messages: HashMap<usize, Vec<Message<T>>>,
    random_value: fn() -> T,
) -> (BroadcastValue<T>, HashMap<usize, Vec<Message<T>>>, Receiver<Message<T>>)
where
    T: Broadcastable,
{
    let round_counter = phase_counter * 3;
    let (results, early_messages, receiver) = round::round(
        round_counter,
        id,
        current_value,
        process_count,
        early_messages,
        receiver,
        sender.clone(),
    );
    current_value = stage_one_selection_protocol(results);


    let (results, early_messages, receiver) = round::round(
        round_counter + 1,
        id,
        current_value.clone(),
        process_count,
        early_messages,
        receiver,
        sender.clone(),
    );
    current_value = stage_two_selection_protocol(results, current_value);


    let (results, early_messages, receiver) = round::round(
        round_counter + 2,
        id,
        current_value.clone(),
        process_count,
        early_messages,
        receiver,
        sender,
    );

    (stage_three_selection_protocol(results, current_value, faulty_count, random_value), early_messages, receiver)
}

fn stage_one_selection_protocol<T>(mut results: Vec<BroadcastValue<T>>) -> BroadcastValue<T>
where
    T: Broadcastable,
{
    let mut results_count = HashMap::new();
    let mut max_result = results.pop().unwrap();
    let mut max_count = 1;
    for result in results {
        let entry = results_count.entry(result.value.clone()).or_insert(0);
        *entry += 1;

        if *entry > max_count {
            max_result = result;
            max_count = *entry;
        }
    }
    max_result
}

fn stage_two_selection_protocol<T>(
    mut results: Vec<BroadcastValue<T>>,
    current_value: BroadcastValue<T>,
) -> BroadcastValue<T>
where
    T: Broadcastable,
{
    let mut results_count = HashMap::new();
    let mut max_result_value = results.pop().unwrap().value;
    let mut max_count = 1;
    for result in &results {
        let entry = results_count.entry(result.value.clone()).or_insert(0);
        *entry += 1;

        if *entry > max_count {
            max_result_value = result.value.clone();
            max_count = *entry;
        }
    }

    if max_count > results.len() / 2 {
        BroadcastValue::new(max_result_value, true)
    } else {
        current_value
    }
}

fn stage_three_selection_protocol<T>(
    mut results: Vec<BroadcastValue<T>>,
    current_value: BroadcastValue<T>,
    faulty_count: usize,
    default: fn() -> T,
) -> BroadcastValue<T>
where
    T: Broadcastable,
{
    let mut results_count = HashMap::new();
    let mut max_result_value = results.pop().unwrap().value;
    let mut max_count = 1;
    for result in &results {
        let entry = results_count.entry(result.value.clone()).or_insert(0);
        *entry += 1;

        if *entry > max_count {
            max_result_value = result.value.clone();
            max_count = *entry;
        }
    }

    if max_count > 2 * faulty_count {
        BroadcastValue::new(max_result_value, true)
    } else if max_count > faulty_count {
        BroadcastValue::new(max_result_value, false)
    } else {
        BroadcastValue::new((default)(), false)
    }
}
