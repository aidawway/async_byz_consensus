use crate::{
    broadcast::BroadcastValue,
    util::{self, Broadcastable},
    validation::ValidatedMessageSet,
};

pub fn selection_protocol<T>(
    round: usize,
    process_count: usize,
    validated: &ValidatedMessageSet<T>,
) -> Option<BroadcastValue<T>>
where
    T: Broadcastable,
{
    match round % 3 {
        0 => round_one_selection_protocol(validated),
        1 => round_two_selection_protocol(process_count, validated),
        2 => round_three_selection_protocol(process_count, validated),
        _ => unreachable!(),
    }
}

fn round_one_selection_protocol<T>(validated: &ValidatedMessageSet<T>) -> Option<BroadcastValue<T>>
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

fn round_two_selection_protocol<T>(
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

fn round_three_selection_protocol<T>(
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
