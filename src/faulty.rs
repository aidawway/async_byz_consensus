use crossbeam::channel::Receiver;

use crate::{broadcast::{BroadcastSender, BroadcastValue}, messaging::{Message, MessageType}, util::{Broadcastable, NetworkInfo}};





pub fn faulty_process<T>(repeated_value: BroadcastValue<T>, network: NetworkInfo<T>) -> bool
where T: Broadcastable
{


    let mut round_count = 0;
    let receiver = network.receiver;
    let process_count = network.senders.len();

    let sender = BroadcastSender::new(network.senders);

    loop {
        let message = receiver.recv().unwrap();

        if message.round == round_count {

            for id in 0..process_count {
                sender.send(Message::new(message.round, id, repeated_value.clone(), MessageType::Initiate));
                sender.send(Message::new(message.round, id, repeated_value.clone(), MessageType::Echo));
                sender.send(Message::new(message.round, id, repeated_value.clone(), MessageType::Ready));
            }
            round_count += 1;
        }
    }
    false
}