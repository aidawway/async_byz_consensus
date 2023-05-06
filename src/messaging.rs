
#[derive(Clone)]
pub struct Message<T> {
    pub broadcast_source_id: usize,
    pub message_type: MessageType,
    pub value: T,
    pub decided: bool,
}

impl<T> Message<T> {
    pub fn new(broadcast_source_id: usize, value: T, decided: bool, message_type: MessageType) -> Message<T> {
        Message {
            broadcast_source_id,
            message_type,
            value,
            decided,
        }
    }
}

#[derive(Clone)]
pub enum MessageType {
    Initiate,
    Echo,
    Ready,
}