use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLatestVer {
    release: Arc<str>,
    snapshot: Arc<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersion {
    pub id: Arc<str>,
    #[serde(rename = "type")]
    pub version_type: Arc<str>,
    pub url: Arc<str>,
    pub time: Arc<str>,
    #[serde(rename = "releaseTime")]
    pub release_time: Arc<str>,
    pub sha1: Arc<str>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionManifest {
    pub release: MinecraftLatestVer,
    pub versions: MinecraftVersion,
}

impl MinecraftVersionManifest {
    pub fn new(version_json: impl AsRef<Path>) -> Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .open(version_json.as_ref())
            .with_context(|| {
                format!(
                    "Failed to read version manifest from: {:#?}",
                    version_json.as_ref()
                )
            })?;

        let mut de = Deserializer::from_reader(f);
        let mvm = Self::deserialize(&mut de).with_context(|| {
            format!(
                "Failed to deserialize version manifest from: {:#?}",
                version_json.as_ref()
            )
        })?;

        Ok(mvm)
    }
}
