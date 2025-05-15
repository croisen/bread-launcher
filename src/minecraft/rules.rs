use serde::{Deserialize, Serialize};
use std::env::consts;
use std::sync::Arc;

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
    VecVal(Vec<Arc<str>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MinecraftRuleOs {
    OSName { name: Arc<str> },
    OSArch { arch: Arc<str> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftRule {
    pub action: Arc<str>, // allow | disallow
    pub features: Option<MinecraftRuleFeatures>,
    pub os: Option<MinecraftRuleOs>,
    pub value: Option<MinecraftRuleValue>,
}

impl MinecraftRule {
    pub fn is_needed(&self) -> bool {
        let b = if let Some(os) = &self.os {
            return match os {
                MinecraftRuleOs::OSName { name } => name.eq_ignore_ascii_case(consts::OS),
                MinecraftRuleOs::OSArch { arch } => arch.eq_ignore_ascii_case(consts::ARCH),
            };
        } else {
            false
        };

        if self.action.eq_ignore_ascii_case("disallow") {
            return !b;
        }

        true
    }
}
