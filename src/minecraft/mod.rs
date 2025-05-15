use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

mod arguments;
mod downloads;
mod java_version;
mod libraries;
mod rules;
mod version_manifest;

pub use arguments::MinecraftArgument;
pub use downloads::MinecraftDownload;
pub use java_version::MinecraftJavaVersion;
pub use libraries::MinecraftLibrary;
pub use rules::MinecraftRule;
use serde_json::Deserializer;
pub use version_manifest::MinecraftVersionManifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minecraft {
    arguments: Arc<MinecraftArgument>,
    downloads: Arc<MinecraftDownload>,
    #[serde(rename = "javaVersion")]
    java_version: Arc<MinecraftJavaVersion>,
    libraries: Arc<Vec<MinecraftLibrary>>,

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

        Ok(m)
    }

    pub async fn download_libs(&self, cache_dir: impl AsRef<Path>) -> Result<()> {
        for lib in self.libraries.iter() {}

        Ok(())
    }
}
