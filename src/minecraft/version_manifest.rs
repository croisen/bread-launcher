use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use tokio::fs::File as TkFile;

use crate::utils;

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

impl MinecraftVersion {
    pub async fn download(&self, cl: &Client, root_dir: impl AsRef<Path>) -> Result<PathBuf> {
        let mut p = root_dir.as_ref().join("cache");
        p.push(self.id.as_ref());
        utils::download::download_with_sha(
            cl,
            &p,
            "client.json",
            &self.url.clone(),
            &self.sha1.clone(),
            true,
            1,
        )
        .await?;

        Ok(p)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionManifest {
    pub latest: MinecraftLatestVer,
    pub versions: Vec<MinecraftVersion>,
}

impl MinecraftVersionManifest {
    pub async fn new(cl: &Client, root_dir: impl AsRef<Path>) -> Result<Self> {
        let version_json = root_dir.as_ref().join("version_manifest_v2.json");
        if !version_json.is_file() {
            Self::download(cl, &root_dir).await?;
        }

        let f = TkFile::open(&version_json).await.with_context(|| {
            format!("Failed to read version manifest from: {:#?}", &version_json)
        })?;
        let mut de = Deserializer::from_reader(f.into_std().await);
        let mvm = Self::deserialize(&mut de)?;
        Ok(mvm)
    }

    pub async fn download(cl: &Client, root_dir: impl AsRef<Path>) -> Result<()> {
        let _ = utils::download::download(
            cl,
            root_dir,
            "version_manifest_v2.json",
            &Arc::from("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"),
        )
        .await;

        Ok(())
    }
}
