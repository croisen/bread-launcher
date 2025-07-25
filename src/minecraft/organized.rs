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

        for ver in &mvm.versions {
            match ver.version_type.as_ref() {
                "release" => {
                    release.push(ver.clone());
                }
                "snapshot" => {
                    snapshot.push(ver.clone());
                }
                "old_beta" => {
                    beta.push(ver.clone());
                }
                "old_alpha" => {
                    alpha.push(ver.clone());
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

    pub fn renew(&self, cl: &Client) -> Result<Self> {
        let mvm = MinecraftVersionManifest::new(cl)?;
        Ok(Self::new(&mvm))
    }
}
