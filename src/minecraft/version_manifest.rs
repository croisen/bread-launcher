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
    pub async fn download(&self, cl: &Client, appdir: impl AsRef<Path>) -> Result<PathBuf> {
        let mut p = appdir.as_ref().join("cache");
        p.push(self.id.as_ref());
        utils::download::download_with_sha(cl, &p, "client.json", &self.url, &self.sha1, true, 1)
            .await?;

        Ok(p)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionManifest {
    pub latest: MinecraftLatestVer,
    pub versions: Vec<Arc<MinecraftVersion>>,
}

impl MinecraftVersionManifest {
    pub async fn new(cl: &Client, appdir: impl AsRef<Path> + Send + Sync) -> Result<Self> {
        let version_json = appdir.as_ref().join("version_manifest_v2.json");
        if !version_json.is_file() {
            Self::download(cl, &appdir).await?;
        }

        let f = TkFile::open(&version_json).await.with_context(|| {
            format!("Failed to read version manifest from: {:#?}", &version_json)
        })?;
        let mut de = Deserializer::from_reader(f.into_std().await);
        let mvm = Self::deserialize(&mut de)?;
        Ok(mvm)
    }

    pub async fn download(cl: &Client, appdir: impl AsRef<Path> + Send + Sync) -> Result<()> {
        utils::download::download(
            cl,
            appdir,
            "version_manifest_v2.json",
            "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
            1,
        )
        .await?;

        Ok(())
    }
}
