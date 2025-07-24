use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub jvm_ram: usize, // in MB
}

impl Default for Settings {
    fn default() -> Self {
        Self { jvm_ram: 2048 }
    }
}
