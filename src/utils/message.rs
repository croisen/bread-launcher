use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Message {
    Downloading(String),
    Message(String),
    Errored(String),
}
