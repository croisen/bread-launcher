use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::minecraft::MinecraftVersionManifest;
use crate::minecraft::version_manifest::MinecraftVersion;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct MVOrganized {
    pub release: Vec<Arc<MinecraftVersion>>,
    pub snapshot: Vec<Arc<MinecraftVersion>>,
    pub beta: Vec<Arc<MinecraftVersion>>,
    pub alpha: Vec<Arc<MinecraftVersion>>,
}

impl MVOrganized {
    pub fn new(mvm: &MinecraftVersionManifest) -> Self {
        let mut release = Vec::new();
        let mut snapshot = Vec::new();
        let mut beta = Vec::new();
        let mut alpha = Vec::new();
        log::info!("Organizing minecraft versions...");
        log::info!("Version count: {}", mvm.versions.len());

        for ver in &mvm.versions {
            match ver.version_type.as_ref() {
                "release" => {
                    release.push(ver.clone());
                    // log::info!("Release found: {} Vec len: {}", ver.id, release.len());
                }
                "snapshot" => {
                    snapshot.push(ver.clone());
                    // log::info!("Snapshot found: {} Vec len: {}", ver.id, snapshot.len());
                }
                "old_beta" => {
                    beta.push(ver.clone());
                    // log::info!("Beta found: {} Vec len: {}", ver.id, beta.len());
                }
                "old_alpha" => {
                    alpha.push(ver.clone());
                    // log::info!("Alpha found: {} Vec len: {}", ver.id, alpha.len());
                }
                _ => {
                    log::error!("Unknown version: {} {}", ver.id, ver.version_type);
                }
            }
        }

        Self {
            release,
            snapshot,
            beta,
            alpha,
        }
    }

    pub fn renew(&self, cl: &Client, appdir: impl AsRef<Path>) -> Result<Self> {
        let mvm = MinecraftVersionManifest::new(cl, appdir.as_ref())?;
        Ok(Self::new(&mvm))
    }
}
