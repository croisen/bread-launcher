use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
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

        Ok(m)
    }

    pub async fn download(&self, cl: &Client, cache_dir: impl AsRef<Path>) -> Result<()> {
        self.downloads.download(&cl, cache_dir.as_ref()).await?;
        for lib in &self.libraries {
            lib.download(&cl, cache_dir.as_ref()).await?;
        }

        Ok(())
    }

    /*
     * The way faster implementation but it spams the mojang servers so
     * I kinda got blocked due to it
    pub async fn download(&self, cl: &Client, cache_dir: impl AsRef<Path>) -> Result<()> {
        use anyhow::anyhow;
        use tokio::task::JoinHandle;

        let mut handles: Vec<JoinHandle<Result<()>>> = vec![];
        let cd = Arc::new(cache_dir.as_ref().to_path_buf());
        let cd1 = cd.clone();
        let cl1 = cl.clone();
        let downloads = self.downloads.clone();
        handles.push(tokio::spawn(async move {
            downloads.download(&cl1, cd1.as_ref()).await?;
            Ok(())
        }));

        for lib in &self.libraries {
            let lib2 = lib.clone();
            let cl2 = cl.clone();
            let cd2 = cd.clone();
            handles.push(tokio::spawn(async move {
                lib2.download(&cl2, cd2.as_ref()).await?;
                Ok(())
            }));
        }

        let mut err = false;
        for handle in handles {
            if let Err(e) = handle.await? {
                log::error!("{e:?}");
                err = true;
            }
        }

        if !err {
            Ok(())
        } else {
            Err(anyhow!("Encountered error in async downloads..."))
        }
    }
    */
}
