use std::thread;

use crossbeam::channel;
use rand::Rng;
use util::{Broadcastable, NetworkInfo};

use crate::broadcast::BroadcastValue;

mod broadcast;
mod byz_protocol;
mod faulty;
mod messaging;
mod phase;
mod round;
mod selection_protocol;
mod util;
mod validation;

//TODO do threads terminate after deciding or only at end of the phase? Or only when everyone has decided?
fn main() {
    let process_count = 100;
    let faulty_count = util::faulty_count(process_count);
    let good_count = process_count - faulty_count;

    let mut senders = Vec::with_capacity(process_count);
    let mut receivers = Vec::with_capacity(process_count);
    for process_id in 0..process_count {
        let (s, r) = channel::unbounded();
        senders.push(s);
        receivers.push(r);
    }

    let mut join_handles = Vec::with_capacity(process_count);
    for process_id in 0..process_count {
        let senders_clone = senders.clone();
        let receiver = receivers.pop().unwrap();
        join_handles.push(thread::spawn(move || {
            byz_protocol::consensus_protocol(
                true,
                // random_boolean(),
                random_boolean,
                NetworkInfo::new(process_id, senders_clone, receiver),
            )
        }))
    }
    for process_id in process_count..process_count + faulty_count {
        let senders_clone = senders.clone();

        if let Some(receiver) = receivers.pop() {
            join_handles.push(thread::spawn(move || {
                faulty::faulty_process(
                    BroadcastValue::new(false, true),
                    NetworkInfo::new(process_id, senders_clone, receiver),
                )
            }));
        }
    }

    for handle in join_handles.into_iter() {
        let result = handle.join().unwrap();
        println!("Agreed on {result}");
    }
    println!("Done");
}

impl Broadcastable for bool {}

fn random_boolean() -> bool {
    rand::thread_rng().gen_bool(0.5)
}
