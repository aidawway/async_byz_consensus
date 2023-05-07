use std::{hash, fmt::Debug};

use crossbeam::channel::{Receiver, Sender};

use crate::messaging::Message;

pub trait Broadcastable: Clone + Eq + Ord + hash::Hash + Send + Debug+ 'static {}

pub struct NetworkInfo<T> {
    pub id: usize,
    pub senders: Vec<Sender<Message<T>>>,
    pub receiver: Receiver<Message<T>>,
}

impl<T> NetworkInfo<T> {
    pub fn new(
        id: usize,
        senders: Vec<Sender<Message<T>>>,
        receiver: Receiver<Message<T>>,
    ) -> NetworkInfo<T> {
        NetworkInfo {
            id,
            senders,
            receiver,
        }
    }
}

//TODO avoid recalculating
pub fn faulty_count(process_count: usize) -> usize {
    // faulty count < process_count / 3
    if process_count % 3 == 0 {
        process_count / 3 - 1
    } else {
        process_count / 3
    }
}
