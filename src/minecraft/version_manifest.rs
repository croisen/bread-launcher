use std::fmt::Debug;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

use crate::utils;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLatestVer {
    release: Arc<str>,
    snapshot: Arc<str>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
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
    pub fn download(&self, cl: &Client, appdir: impl AsRef<Path>) -> Result<PathBuf> {
        let ver = format!("{}.json", self.id.as_ref());
        let mut p = appdir.as_ref().join("minecraft_cache");
        p.push("versions");
        utils::download::download_with_sha(cl, &p, ver, &self.url, &self.sha1, 1)?;

        Ok(p)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionManifest {
    pub latest: MinecraftLatestVer,
    pub versions: Vec<Arc<MinecraftVersion>>,
}

impl MinecraftVersionManifest {
    pub fn new(cl: &Client, appdir: impl AsRef<Path> + Send + Sync) -> Result<Self> {
        let version_json = appdir.as_ref().join("version_manifest_v2.json");
        if !version_json.is_file() {
            Self::download(cl, &appdir)?;
        }

        let f = File::open(&version_json).with_context(|| {
            format!("Failed to read version manifest from: {:#?}", &version_json)
        })?;

        let mut de = Deserializer::from_reader(f);
        let mvm = Self::deserialize(&mut de)?;

        Ok(mvm)
    }

    pub fn download(cl: &Client, appdir: impl AsRef<Path> + Send + Sync) -> Result<()> {
        utils::download::download(
            cl,
            appdir,
            "version_manifest_v2.json",
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
            1,
        )?;

        Ok(())
    }
}
