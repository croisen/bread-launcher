use std::fmt::Debug;

use serde::{Deserialize, Serialize};

pub mod forge;
pub mod minecraft;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedVersionManifest {
    pub mc: minecraft::MVOrganized,
    pub forge: forge::ForgeVersionManifest,
}
