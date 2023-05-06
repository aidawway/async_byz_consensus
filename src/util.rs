use std::hash;

pub trait Broadcastable: Clone + Eq + hash::Hash + Send + 'static {}
