use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpmc::Receiver as MultiReceiver;
use std::sync::mpsc::Sender as SingleSender;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

mod libraries;
mod version_manifest;

pub use version_manifest::ForgeVersionManifest;

use crate::init::get_versiondir;
use crate::loaders::minecraft::MinecraftLibrary;
use crate::utils::message::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct Forge {
    minecraft: String,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    libraries: Vec<MinecraftLibrary>,

    #[serde(skip)]
    instance_dir: PathBuf,
}

impl Forge {
    pub fn new(
        instance_dir: impl AsRef<Path>,
        mc_ver: impl AsRef<str>,
        forge_ver: impl AsRef<str>,
    ) -> Result<Self> {
        let json = get_versiondir().join(format!(
            "forge-{}-{}.json",
            mc_ver.as_ref(),
            forge_ver.as_ref()
        ));

        let json = read_to_string(&json).context(format!("Failed to read {json:?}"))?;
        let mut m: Self = from_str(&json)?;
        m.instance_dir = instance_dir.as_ref().to_path_buf();

        Ok(m)
    }

    pub fn download(
        &self,
        cl: Client,
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        rx: MultiReceiver<()>,
    ) -> Result<()> {
        Ok(())
    }
}
