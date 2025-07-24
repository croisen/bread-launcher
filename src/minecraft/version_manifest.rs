use std::fmt::Debug;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::init::get_appdir;
use crate::init::get_versiondir;
use crate::utils;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLatestVer {
    release: Arc<str>,
    snapshot: Arc<str>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl MinecraftVersion {
    pub fn download(&self, cl: &Client) -> Result<()> {
        let ver = format!("{}.json", self.id.as_ref());
        utils::download::download_with_sha(cl, get_versiondir(), ver, &self.url, &self.sha1, 1)?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionManifest {
    pub latest: MinecraftLatestVer,
    pub versions: Vec<Arc<MinecraftVersion>>,
}

impl MinecraftVersionManifest {
    pub fn new(cl: &Client) -> Result<Self> {
        let version_json = get_appdir().join("version_manifest_v2.json");
        if !version_json.is_file() {
            Self::download(cl)?;
        }

        let f = File::open(&version_json).with_context(|| {
            format!("Failed to read version manifest from: {:#?}", &version_json)
        })?;

        let mut de = Deserializer::from_reader(f);
        let mvm = Self::deserialize(&mut de)?;

        Ok(mvm)
    }

    pub fn download(cl: &Client) -> Result<()> {
        utils::download::download(
            cl,
            get_appdir(),
            "version_manifest_v2.json",
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
            1,
        )?;

        Ok(())
    }
}
