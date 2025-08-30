use serde::{Deserialize, Serialize};

use crate::minecraft::MinecraftRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    ArgStr(String),
    ArgRul { rules: Vec<MinecraftRule> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftArgument {
    game: Vec<Argument>,
    jvm: Vec<Argument>,
}
