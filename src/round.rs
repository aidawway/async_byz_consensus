use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    thread,
};

use crossbeam::{
    channel::{self, Receiver, Sender},
    select,
};

use crate::{
    broadcast::{self, Broadcast, BroadcastSender, BroadcastValue},
    messaging::Message,
    util::Broadcastable,
    validation::ValidatedMessageSet,
};

pub fn round<T>(
    round: usize,
    process_id: usize,
    initial_value: BroadcastValue<T>,
    process_count: usize,
    early_messages: HashMap<usize, Vec<Message<T>>>,
    previously_validated: ValidatedMessageSet<T>,
    receiver: Receiver<Message<T>>,
    sender: BroadcastSender<T>,
) -> (
    ValidatedMessageSet<T>,
    HashMap<usize, Vec<Message<T>>>,
    Receiver<Message<T>>,
)
where
    T: Broadcastable,
{
    //Initiate broadcast for each process
    let mut internal_senders = Vec::with_capacity(process_count);
    let mut handles = Vec::with_capacity(process_count);

    for index in 0..process_count {
        let (s, r) = channel::unbounded();
        internal_senders.push(s);

        let sender_clone = sender.clone();
        let bcast = Broadcast::new(round, r, sender.clone(), process_count);
        if index == process_id {
            //Broadcast initialize message
            let initial_value_clone = initial_value.clone();

            handles.push(thread::spawn(move || {
                bcast.local_broadcast(process_id, initial_value_clone)
            }))
        } else {
            handles.push(thread::spawn(move || bcast.broadcast_protocol()));
        }
    }

    // Start router to forward external messages to proper thread (necessary due to constraining channels to mpsc functionality)
    let (router_sender, router_receiver) = channel::bounded(1);
    let router_handle = thread::spawn(move || {
        route_messages(
            process_id,
            round,
            early_messages,
            receiver,
            internal_senders,
            router_receiver,
        )
    });

    // Collect results
    let mut validated = ValidatedMessageSet::new();
    for handle in handles {
        let accepted = handle.join().unwrap();
        if previously_validated.validate(round, process_count, &accepted) {
            validated.add(accepted);
        }
    }

    //Terminate router
    router_sender.send("exit".to_string());
    let (early_messages, receiver) = router_handle.join().unwrap();

    (validated, early_messages, receiver)
}

fn route_messages<T>(
    id: usize,
    round: usize,
    mut early_messages: HashMap<usize, Vec<Message<T>>>,
    external_receiver: Receiver<Message<T>>,
    internal_senders: Vec<Sender<Message<T>>>,
    terminate_receiver: Receiver<String>,
) -> (HashMap<usize, Vec<Message<T>>>, Receiver<Message<T>>)
where
    T: Broadcastable,
{
    //Forward messages that arrived before the local round started
    if let Some(messages) = early_messages.remove(&round) {
        for message in messages {
            forward(message, &internal_senders);
        }
    }

    loop {
        //TODO end condition?

        select! {
            recv(external_receiver) -> message => {
                let message = message.unwrap();
                if message.round == round {
                    forward(message, &internal_senders)
                }
                else if message.round > round {
                    let entry = early_messages.entry(message.round).or_insert(Vec::new());
                    entry.push(message);
                }
            }
            recv(terminate_receiver) -> _ => {
            return (early_messages, external_receiver)
            }
        }
    }
}

fn forward<T>(message: Message<T>, senders: &Vec<Sender<Message<T>>>) {
    if let Some(sender) = senders.get(message.broadcast_source_id) {
        sender.send(message);
    } else {
        panic!()
    }
}
