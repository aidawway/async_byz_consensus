use std::{cmp, collections::HashMap, hash};

use crate::{
    messaging::{Message, MessageType},
    util::Broadcastable,
};
use crossbeam::channel::{Receiver, Sender};
use log::error;

#[derive(Clone)]
pub struct BroadcastSender<T> {
    senders: Vec<Sender<Message<T>>>,
}

#[derive(Clone, Debug, Eq, PartialEq, hash::Hash)]
pub struct BroadcastValue<T> {
    pub value: T,
    pub decided: bool,
}

impl<T> BroadcastValue<T> {
    pub fn new(value: T, decided: bool) -> BroadcastValue<T> {
        BroadcastValue { value, decided }
    }
}

impl<T> BroadcastSender<T>
where
    T: Broadcastable,
{
    pub fn new(senders: Vec<Sender<Message<T>>>) -> BroadcastSender<T> {
        BroadcastSender { senders }
    }

    pub fn send(&self, msg: Message<T>) {
        for sender in &self.senders {
            sender.send(msg.clone());
        }
    }
}

pub struct Broadcast<T> {
    receiver: Receiver<Message<T>>,
    sender: BroadcastSender<T>,
    round: usize,
    process_count: usize,
}

impl<T> Broadcast<T>
where
    T: Broadcastable,
{
    pub fn new(
        round: usize,
        receiver: Receiver<Message<T>>,
        sender: BroadcastSender<T>,
        process_count: usize,
    ) -> Broadcast<T> {
        Broadcast {
            round, //TODO index from 1 or 0
            receiver,
            sender,
            process_count,
        }
    }
    pub fn local_broadcast(
        &self,
        broadcast_source_id: usize,
        initial_value: BroadcastValue<T>,
    ) -> BroadcastValue<T>
    where
        T: Broadcastable,
    {
        //Send initial message
        self.sender.send(Message::new(
            self.round,
            broadcast_source_id,
            initial_value,
            MessageType::Initiate,
        ));
        self.broadcast_protocol()
    }

    pub fn broadcast_protocol(&self) -> BroadcastValue<T>
    where
        T: Broadcastable,
    {
        let faulty_count = if self.process_count % 3 == 0 {
            self.process_count / 3 - 1
        } else {
            self.process_count / 3
        };
        let mut echo_count = HashMap::new();
        let mut ready_count = HashMap::new();

        let (echo_count, ready_count) =
            self.broadcast_stage_one(faulty_count, echo_count, ready_count);
        let ready_count = self.broadcast_stage_two(faulty_count, echo_count, ready_count);

        self.broadcast_stage_three(faulty_count, ready_count)
    }

    fn broadcast_stage_one(
        &self,
        faulty_count: usize,
        mut echo_count: HashMap<T, usize>,
        mut ready_count: HashMap<T, usize>,
    ) -> (HashMap<T, usize>, HashMap<T, usize>)
    where
        T: Broadcastable,
    {
        let mut initialized = false;

        while !initialized {
            let Message {
                round,
                broadcast_source_id,
                message_type,
                value,
            } = self.receiver.recv().unwrap();
            match message_type {
                MessageType::Initiate => {
                    initialized = true;
                    self.sender.send(Message::new(
                        round,
                        broadcast_source_id,
                        value,
                        MessageType::Echo,
                    ));
                }
                MessageType::Echo => {
                    let entry = echo_count.entry(value.value.clone()).or_insert(0);
                    *entry += 1;
                    if *entry >= (self.process_count + faulty_count) / 2 {
                        initialized = true;
                        self.sender.send(Message::new(
                            round,
                            broadcast_source_id,
                            value,
                            MessageType::Echo,
                        ));
                    }
                }
                MessageType::Ready => {
                    let entry = ready_count.entry(value.value.clone()).or_insert(0);
                    *entry += 1;
                    if *entry >= (self.process_count + faulty_count) / 2 {
                        initialized = true;
                        self.sender.send(Message::new(
                            round,
                            broadcast_source_id,
                            value,
                            MessageType::Echo,
                        ));
                    }
                }
            }
        }
        (echo_count, ready_count)
    }

    fn broadcast_stage_two(
        &self,
        faulty_count: usize,
        mut echo_count: HashMap<T, usize>,
        mut ready_count: HashMap<T, usize>,
    ) -> HashMap<T, usize>
    where
        T: Broadcastable,
    {
        let mut readied = false;

        while !readied {
            let Message {
                round,
                broadcast_source_id,
                message_type,
                value,
            } = self.receiver.recv().unwrap();
            match message_type {
                MessageType::Initiate => (), //Discard message
                MessageType::Echo => {
                    let entry = echo_count.entry(value.value.clone()).or_insert(0);
                    *entry += 1;
                    if *entry >= (self.process_count + faulty_count) / 2 {
                        readied = true;
                        self.sender.send(Message::new(
                            round,
                            broadcast_source_id,
                            value,
                            MessageType::Ready,
                        ));
                    }
                }
                MessageType::Ready => {
                    let entry = ready_count.entry(value.value.clone()).or_insert(0);
                    *entry += 1;
                    if *entry >= (self.process_count + faulty_count) / 2 {
                        readied = true;
                        self.sender.send(Message::new(
                            round,
                            broadcast_source_id,
                            value,
                            MessageType::Ready,
                        ));
                    }
                }
            }
        }

        ready_count
    }

    fn broadcast_stage_three(
        &self,
        faulty_count: usize,
        mut ready_count: HashMap<T, usize>,
    ) -> BroadcastValue<T>
    where
        T: Broadcastable,
    {
        let mut result = None;
        while result.is_none() {
            let Message {
                round,
                broadcast_source_id,
                message_type,
                value,
            } = self.receiver.recv().unwrap();

            match message_type {
                MessageType::Initiate | MessageType::Echo => (), //Discard message
                MessageType::Ready => {
                    let entry = ready_count.entry(value.value.clone()).or_insert(0);
                    *entry += 1;
                    if *entry >= 2 * faulty_count + 1 {
                        result = Some(value);
                    }
                }
            }
        }
        result.unwrap()
    }
}
