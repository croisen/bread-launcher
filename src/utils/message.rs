use std::sync::mpmc::{channel, Receiver, Sender};
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

// Intentional memory leak, the OS can take care of it faster tbh
// Considering that I cannot put channels in a serde struct,
// this is my best solution for now
static CHANNEL: LazyLock<(Sender<Message>, Receiver<Message>)> = LazyLock::new(|| channel());

pub fn get_sender() -> Sender<Message> {
    CHANNEL.0.clone()
}

pub fn get_receiver() -> Receiver<Message> {
    CHANNEL.1.clone()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    Downloading(String),
    Message(String),
    Errored(String),
}
