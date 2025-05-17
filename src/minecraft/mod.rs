use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

mod arguments;
mod assets;
mod downloads;
mod java_version;
mod libraries;
mod organized;
mod rules;
mod version_manifest;

pub use arguments::MinecraftArgument;
pub use assets::MinecraftAsset;
pub use downloads::MinecraftDownload;
pub use java_version::MinecraftJavaVersion;
pub use libraries::MinecraftLibrary;
pub use organized::MVOrganized;
pub use rules::MinecraftRule;
pub use version_manifest::MinecraftVersionManifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minecraft {
    arguments: Option<Arc<MinecraftArgument>>,
    #[serde(rename = "minecraftArguments")]
    minecraft_arguments: Option<Arc<arguments::Argument>>,
    #[serde(rename = "assetIndex")]
    asset_index: Arc<MinecraftAsset>,
    downloads: Arc<MinecraftDownload>,
    #[serde(default, rename = "javaVersion")]
    java_version: Arc<MinecraftJavaVersion>,
    libraries: Vec<Arc<MinecraftLibrary>>,

    id: Arc<str>,
    #[serde(rename = "mainClass")]
    main_class: Arc<str>,
    #[serde(rename = "minimumLauncherVersion")]
    minimum_launcher_version: usize,
    #[serde(rename = "releaseTime")]
    release_time: Arc<str>,
    time: Arc<str>,
    #[serde(rename = "type")]
    release_type: Arc<str>,
}

impl Minecraft {
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let json = cache_dir.as_ref().join("client.json");
        let f = OpenOptions::new()
            .read(true)
            .open(&json)
            .with_context(|| format!("Failed to open {:#?}", &json))?;
        let mut de = Deserializer::from_reader(f);
        let m =
            Self::deserialize(&mut de).with_context(|| format!("Failed to parse {:#?}", &json))?;

        log::info!("MC Version:   {}", m.id.as_ref());
        log::info!("Java Version: {:?}", m.java_version.as_ref());

        Ok(m)
    }

    pub async fn download(&self, cl: &Client, cache_dir: impl AsRef<Path>) -> Result<()> {
        log::info!("Checking client main files");
        self.downloads.download(&cl, cache_dir.as_ref()).await?;
        log::info!("Checking client assets");
        self.asset_index.download(&cl, cache_dir.as_ref()).await?;
        log::info!("Checking java runtime environment");
        let jre = self
            .java_version
            .download(
                &cl,
                cache_dir
                    .as_ref()
                    .to_path_buf()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap(),
            )
            .await?;

        log::info!("JRE path: {:?}", jre);
        log::info!("Checking client libraries");
        for lib in &self.libraries {
            lib.download(&cl, cache_dir.as_ref()).await?;
        }

        Ok(())
    }
}
