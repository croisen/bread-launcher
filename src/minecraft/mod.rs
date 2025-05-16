use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;

mod arguments;
mod downloads;
mod java_version;
mod libraries;
mod organized;
mod rules;
mod version_manifest;

pub use arguments::MinecraftArgument;
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
    downloads: Arc<MinecraftDownload>,
    #[serde(rename = "javaVersion")]
    java_version: Option<Arc<MinecraftJavaVersion>>,
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

    pub async fn download(&self, cl: &Client, cache_dir: impl AsRef<Path>) -> Result<()> {
        self.downloads.download(cl, &cache_dir).await?;
        let cd = Arc::new(cache_dir.as_ref().to_path_buf());
        let libs = self.libraries.clone();
        let mut handles = vec![];
        for lib in libs.iter() {
            let cl2 = cl.clone();
            let cd2 = cd.clone();
            handles.push(async move {
                if let Err(e) = lib.download(&cl2, cd2.as_ref()).await {
                    log::error!("{e:?}");
                }
            });
        }

        for handle in handles {
            handle.await;
        }

        Ok(())
    }
}
