use std::error::Error;
use std::fmt::Display;
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Deserializer;

use crate::utils;

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftJson {
    pub libraries: Vec<MinecraftLibs>,
    pub id: String, // Version i.e. 1.21.5
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: usize,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "type")]
    pub reltype: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JavaVersion {
    component: Arc<str>,
    #[serde(rename = "majorVersion")]
    major_version: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftLibs {
    pub downloads: MinecraftLibDownload,
    pub name: Arc<str>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftLibDownload {
    pub artifact: MinecraftArtifact,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftArtifact {
    pub path: Arc<str>,
    pub sha1: String,
    pub size: usize,
    pub url: Arc<str>,
}

impl MinecraftJson {
    /// ```
    /// cache_dir: Path - from the output of crate::launcher::Minecraft download()
    /// ```
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let json = cache_dir.as_ref().join("client.json");
        let mjf = OpenOptions::new().read(true).open(json)?;
        let mj = Self::deserialize(&mut Deserializer::from_reader(mjf))?;
        Ok(mj)
    }

    /// ```
    /// cl: async Client - (cloning is fine, it's an Arc internally)
    /// cache: Path - ~/.local/share/breadlauncher/cache/{version}
    /// I expect the dir struct to be like this
    /// appdir/cache/version/{libraries/, client.jar, client.json}
    /// and the libraries part is where download_libs goes brrr with async
    /// ```
    pub async fn download_libs(
        &self,
        cl: Client,
        cache: impl AsRef<Path>,
    ) -> Result<(), Box<dyn Error>> {
        let mut handles = vec![];
        for l in &self.libraries {
            let mut libf = cache.as_ref().join("libraries");
            let path = l
                .downloads
                .artifact
                .path
                .split("/")
                .into_iter()
                .collect::<Vec<&str>>();

            let last = path.last().unwrap();
            for i in 0..path.len() - 1 {
                libf.push(path.get(i).unwrap());
            }

            let url = l.downloads.artifact.url.clone();
            handles.push(utils::download(&cl, libf, last, &url).await);
        }

        for handle in handles {
            if let Err(e) = handle.await {
                log::error!("{e}");
            }
        }

        Ok(())
    }
}
