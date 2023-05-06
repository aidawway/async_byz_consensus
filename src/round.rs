use std::thread;

use crossbeam::{
    channel::{self, Receiver, Sender},
    select,
};

use crate::{
    broadcast::{self, BroadcastSender},
    messaging::Message,
    util::Broadcastable,
};

fn round<T>(
    process_id: usize,
    initial_value: T,
    decided: bool,
    process_count: usize,
    receiver: Receiver<Message<T>>,
    sender: BroadcastSender<T>,
) -> (Vec<T>, Receiver<Message<T>>)
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
        if index == process_id {
            //Broadcast initialize message
            let initial_value_clone = initial_value.clone();
            handles.push(thread::spawn(move || {
                broadcast::local_broadcast(
                    process_id,
                    initial_value_clone,
                    decided,
                    process_count,
                    r,
                    sender_clone,
                )
            }))
        } else {
            handles.push(thread::spawn(move || {
                broadcast::broadcast_protocol(process_count, r, sender_clone)
            }));
        }
    }

    // Start router to forward external messages to proper thread (necessary due to constraining channels to mpsc functionality)
    let (router_sender, router_receiver) = channel::bounded(1);
    let router_handle =
        thread::spawn(move || route_messages(receiver, internal_senders, router_receiver));

    // Collect results
    let mut results = Vec::with_capacity(process_count);
    for handle in handles {
        results.push(handle.join().unwrap());
    }

    //Terminate router
    router_sender.send(()).unwrap();
    let receiver = router_handle.join().unwrap();

    (results, receiver)
}

fn route_messages<T>(
    external_receiver: Receiver<Message<T>>,
    internal_senders: Vec<Sender<Message<T>>>,
    terminate_receiver: Receiver<()>,
) -> Receiver<Message<T>> {
    loop {
        //TODO end condition?

        select! {
            recv(external_receiver) -> message => forward(message.unwrap(), &internal_senders),
            recv(terminate_receiver) -> _ => return external_receiver
        }
    }
}

fn forward<T>(message: Message<T>, senders: &Vec<Sender<Message<T>>>) {
    if let Some(sender) = senders.get(message.broadcast_source_id) {
        sender.send(message).unwrap();
    } else {
        panic!()
    }
}
