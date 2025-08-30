use serde::{Deserialize, Serialize};
use std::env::consts;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftRuleFeatures {
    has_custom_resolution: Option<bool>,
    has_quick_plays_support: Option<bool>,
    is_demo_user: Option<bool>,
    is_quick_play_multiplayer: Option<bool>,
    is_quick_play_singleplayer: Option<bool>,
    is_quick_play_realms: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MinecraftRuleValue {
    StringVal(String),
    VecVal(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MinecraftRuleOs {
    OSName { name: String },
    OSArch { arch: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftRule {
    pub action: String, // allow | disallow
    pub features: Option<MinecraftRuleFeatures>,
    pub os: Option<MinecraftRuleOs>,
    pub value: Option<MinecraftRuleValue>,
}

impl MinecraftRule {
    pub fn is_needed(&self) -> bool {
        let harch = if consts::ARCH == "x86_64" {
            "x64"
        } else {
            consts::ARCH
        };

        let needed = if let Some(os) = &self.os {
            match os {
                MinecraftRuleOs::OSName { name } => name.eq_ignore_ascii_case(consts::OS),
                MinecraftRuleOs::OSArch { arch } => arch.eq_ignore_ascii_case(harch),
            }
        } else {
            true
        };

        if self.action.eq_ignore_ascii_case("disallow") {
            !needed
        } else {
            needed
        }
    }
}
