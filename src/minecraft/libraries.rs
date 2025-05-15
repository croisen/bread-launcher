use std::env::consts;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use crate::{minecraft::MinecraftRule, utils};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MinecraftLibArtifact {
    pub path: Arc<str>,
    pub sha1: Arc<str>,
    pub size: usize,
    pub url: Arc<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MinecraftLibDownload {
    #[serde(rename = "artifact")]
    Artifact(MinecraftLibArtifact),
    #[serde(rename = "classifiers")]
    Classifiers {
        #[serde(rename = "natives-linux")]
        natives_linux: Option<MinecraftLibArtifact>,
        #[serde(rename = "natives-windows")]
        natives_windows: Option<MinecraftLibArtifact>,
        #[serde(rename = "natives-windows-32")]
        natives_windows_32: Option<MinecraftLibArtifact>,
        #[serde(rename = "natives-windows-64")]
        natives_windows_64: Option<MinecraftLibArtifact>,
        #[serde(rename = "natives-osx")]
        natives_osx: Option<MinecraftLibArtifact>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLibrary {
    downloads: MinecraftLibDownload,
    name: Arc<str>,
    rules: Option<Arc<Vec<MinecraftRule>>>,
}

impl MinecraftLibrary {
    pub async fn download(&self, cl: Client, lib_dir: impl AsRef<Path>) -> Result<()> {
        if let Some(r) = self.rules.clone() {
            for rule in r.iter() {
                if !rule.is_needed() {
                    return Ok(());
                }
            }
        }

        // split path get last

        // download

        self.check_sha_redownload(cl, lib_dir, 1).await?;
        Ok(())
    }

    async fn check_sha_redownload(
        &self,
        cl: Client,
        lib_dir: impl AsRef<Path>,
        attempts: usize,
    ) -> Result<()> {
        if attempts >= 4 {
            return Err(anyhow!("SHA1 mismatched even with 4 re-downloads"));
        }

        // split path extend lib dir

        // compare_sha1
        self.download(cl, lib_dir.as_ref()).await?;
        Ok(())
    }
}
