use crate::broadcast::BroadcastValue;


#[derive(Clone, Debug)]
pub struct Message<T> {
    pub round: usize,
    pub broadcast_source_id: usize,
    pub message_type: MessageType,
    pub value: BroadcastValue<T>,
}

impl<T> Message<T> {
    pub fn new(round: usize, broadcast_source_id: usize, value:BroadcastValue<T>, message_type: MessageType) -> Message<T> {
        Message {
            round,
            broadcast_source_id,
            message_type,
            value,
        }
    }
}



#[derive(Clone, Debug)]
pub enum MessageType {
    Initiate,
    Echo,
    Ready,
}