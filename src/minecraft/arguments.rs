use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::minecraft::MinecraftRule;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    ArgStr(Arc<str>),
    ArgRul { rules: Arc<Vec<MinecraftRule>> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftArgument {
    game: Arc<Vec<Argument>>,
    jvm: Arc<Vec<Argument>>,
}
