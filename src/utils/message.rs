use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Message {
    Downloading(String),
    Message(String),
    Errored(String),
}

impl Default for Message {
    fn default() -> Self {
        Message::Message("Snooping around I see".to_string())
    }
}
