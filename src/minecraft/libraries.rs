use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::minecraft::MinecraftRule;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MinecraftLibArtifact {
    pub path: Arc<str>,
    pub sha1: Arc<str>,
    pub size: usize,
    pub url: Arc<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLibClassifiers {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLibDownload {
    artifact: Option<MinecraftLibArtifact>,
    classifiers: Option<MinecraftLibClassifiers>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLibrary {
    downloads: MinecraftLibDownload,
    name: Arc<str>,
    rules: Option<Arc<Vec<MinecraftRule>>>,
}

impl MinecraftLibrary {
    pub async fn download(&self, cl: &Client, cache_dir: impl AsRef<Path>) -> Result<()> {
        if let Some(mla) = self.get_artifact() {
            let mut ld = cache_dir.as_ref().join("libraries");
            let v = mla.path.split("/").collect::<Vec<&str>>();
            let file = v.last().unwrap();
            ld.extend(v.iter());
            let _ = ld.pop();
            utils::download::download_with_sha(
                cl,
                &ld,
                file,
                &mla.url.clone(),
                &mla.sha1.clone(),
                true,
                1,
            )
            .await?;
        }

        if let Some(nat) = self.get_native() {
            let mut ld = cache_dir.as_ref().join("libraries");
            let v = nat.path.split("/").collect::<Vec<&str>>();
            let file = v.last().unwrap();
            ld.extend(v.iter());
            let _ = ld.pop();
            utils::download::download_with_sha(
                cl,
                &ld,
                file,
                &nat.url.clone(),
                &nat.sha1.clone(),
                true,
                1,
            )
            .await?;
        }

        Ok(())
    }

    fn get_artifact(&self) -> Option<&MinecraftLibArtifact> {
        if let Some(mla) = &self.downloads.artifact {
            return Some(mla);
        };

        None
    }

    fn get_native(&self) -> Option<&MinecraftLibArtifact> {
        if let Some(cl) = &self.downloads.classifiers {
            #[cfg(target_os = "linux")]
            return cl.natives_linux.as_ref();

            #[cfg(target_os = "windows")]
            if let Some(nw) = &cl.natives_windows {
                return Some(nw);
            } else {
                #[cfg(target_arch = "x86")]
                return cl.natives_windows_32.as_ref();

                #[cfg(target_arch = "x86_64")]
                return cl.natives_windows_64.as_ref();
            }

            #[cfg(target_os = "macos")]
            return cl.natives_osx.as_ref();
        };

        None
    }
}
