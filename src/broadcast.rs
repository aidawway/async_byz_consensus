use std::{cmp, collections::HashMap, hash};

use crossbeam::channel::{Receiver, Sender};

#[derive(Clone)]
struct Message<T> {
    message_type: MessageType,
    value: T,
    decided: bool,
}

impl<T> Message<T> {
    fn new(value: T, decided: bool, message_type: MessageType) -> Message<T> {
        Message {
            message_type,
            value,
            decided,
        }
    }
}

trait Broadcastable: Clone + Eq + hash::Hash {}

struct BroadcastSender<T> {
    senders: Vec<Sender<Message<T>>>,
}

impl<T> BroadcastSender<T>
where
    T: Broadcastable,
{
    fn send(&self, msg: Message<T>) {
        for sender in &self.senders {
            sender.send(msg.clone()).unwrap();
        }
    }
}

#[derive(Clone)]
enum MessageType {
    Initiate,
    Echo,
    Ready,
}

fn local_broadcast<T>(
    initial_value: T,
    decided: bool,
    process_count: usize,
    receiver: &Receiver<Message<T>>,
    sender: &BroadcastSender<T>,
) -> T
where
    T: Broadcastable,
{
    //Send initial message
    sender.send(Message::new(initial_value, decided, MessageType::Initiate));
    broadcast_protocol(process_count, receiver, sender)
}

fn broadcast_protocol<T>(
    process_count: usize,
    receiver: &Receiver<Message<T>>,
    sender: &BroadcastSender<T>,
) -> T
where
    T: Broadcastable,
{
    let faulty_count = if process_count % 3 == 0 {
        process_count / 3 - 1
    } else {
        process_count / 3
    };
    let mut echo_count = HashMap::new();
    let mut ready_count = HashMap::new();

    let (echo_count, ready_count) = broadcast_stage_one(
        process_count,
        faulty_count,
        echo_count,
        ready_count,
        receiver,
        sender,
    );
    let ready_count = broadcast_stage_two(
        process_count,
        faulty_count,
        echo_count,
        ready_count,
        receiver,
        sender,
    );

    broadcast_stage_three(faulty_count, ready_count, receiver)
}

fn broadcast_stage_one<T>(
    process_count: usize,
    faulty_count: usize,
    mut echo_count: HashMap<T, usize>,
    mut ready_count: HashMap<T, usize>,
    receiver: &Receiver<Message<T>>,
    sender: &BroadcastSender<T>,
) -> (HashMap<T, usize>, HashMap<T, usize>)
where
    T: Broadcastable,
{
    let mut initialized = false;

    while !initialized {
        let Message {
            message_type,
            value,
            decided,
        } = receiver.recv().unwrap();
        match message_type {
            MessageType::Initiate => {
                initialized = true;
                sender.send(Message::new(value, decided, MessageType::Echo));
            }
            MessageType::Echo => {
                let entry = echo_count.entry(value.clone()).or_insert(0);
                *entry += 1;
                if *entry >= (process_count + faulty_count) / 2 {
                    initialized = true;
                    sender.send(Message::new(value, decided, MessageType::Echo));
                }
            }
            MessageType::Ready => {
                let entry = ready_count.entry(value.clone()).or_insert(0);
                *entry += 1;
                if *entry >= (process_count + faulty_count) / 2 {
                    initialized = true;
                    sender.send(Message::new(value, decided, MessageType::Echo));
                }
            }
        }
    }

    (echo_count, ready_count)
}

fn broadcast_stage_two<T>(
    process_count: usize,
    faulty_count: usize,
    mut echo_count: HashMap<T, usize>,
    mut ready_count: HashMap<T, usize>,
    receiver: &Receiver<Message<T>>,
    sender: &BroadcastSender<T>,
) -> HashMap<T, usize>
where
    T: Broadcastable,
{
    let mut readied = false;

    while !readied {
        let Message {
            message_type,
            value,
            decided,
        } = receiver.recv().unwrap();
        match message_type {
            MessageType::Initiate => (), //Discard message
            MessageType::Echo => {
                let entry = echo_count.entry(value.clone()).or_insert(0);
                *entry += 1;
                if *entry >= (process_count + faulty_count) / 2 {
                    readied = true;
                    sender.send(Message::new(value, decided, MessageType::Echo));
                }
            }
            MessageType::Ready => {
                let entry = ready_count.entry(value.clone()).or_insert(0);
                *entry += 1;
                if *entry >= (process_count + faulty_count) / 2 {
                    readied = true;
                    sender.send(Message::new(value, decided, MessageType::Echo));
                }
            }
        }
    }

    ready_count
}

fn broadcast_stage_three<T>(
    faulty_count: usize,
    mut ready_count: HashMap<T, usize>,
    receiver: &Receiver<Message<T>>,
) -> T
where
    T: Broadcastable,
{
    let mut result = None;

    while result.is_none() {
        let Message {
            message_type,
            value,
            decided,
        } = receiver.recv().unwrap();
        match message_type {
            MessageType::Initiate | MessageType::Echo => (), //Discard message
            MessageType::Ready => {
                let entry = ready_count.entry(value.clone()).or_insert(0);
                *entry += 1;
                if *entry >= 2 * faulty_count + 1 {
                    result = Some(value);
                }
            }
        }
    }

    result.unwrap()
}