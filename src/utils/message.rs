use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Message {
    Downloading(String),
    Msg(String),
    Errored(String),
}

impl Message {
    pub fn downloading(msg: impl AsRef<str>) -> Self {
        Message::Downloading(msg.as_ref().into())
    }

    pub fn msg(msg: impl AsRef<str>) -> Self {
        Message::Msg(msg.as_ref().into())
    }

    pub fn errored(msg: impl AsRef<str>) -> Self {
        Message::Errored(msg.as_ref().into())
    }
}

impl Default for Message {
    fn default() -> Self {
        Message::msg("Snooping around I see")
    }
}
