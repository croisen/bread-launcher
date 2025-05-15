use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftJavaVersion {
    component: Arc<str>,
    #[serde(rename = "majorVersion")]
    major_version: usize,
}
