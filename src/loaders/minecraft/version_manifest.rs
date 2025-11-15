use std::fmt::Debug;
use std::fs::read_to_string;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::init::{L_MINECRAFT_VER, R_MINECRAFT_VER, get_appdir, get_versiondir};
use crate::utils::download::{download, download_with_sha1};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLatestVer {
    release: Arc<str>,
    snapshot: Arc<str>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MinecraftVersion {
    pub id: Arc<str>,
    #[serde(rename = "type")]
    pub version_type: Arc<str>,
    pub url: Arc<str>,
    pub time: Arc<str>,
    #[serde(rename = "releaseTime")]
    pub release_time: Arc<str>,
    pub sha1: Arc<str>,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: usize,
}

impl MinecraftVersion {
    pub fn download(&self, cl: Client) -> Result<()> {
        let ver = format!("{}.json", self.id.as_ref());
        download_with_sha1(&cl, get_versiondir(), ver, &self.url, &self.sha1, 1)?;

        Ok(())
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionManifest {
    pub latest: MinecraftLatestVer,
    pub versions: Vec<Arc<MinecraftVersion>>,
}

impl MinecraftVersionManifest {
    pub fn new(cl: Client) -> Result<Self> {
        let mut minecraft_vers = get_appdir();
        minecraft_vers.extend(["loaders", L_MINECRAFT_VER]);
        if !minecraft_vers.is_file() {
            let _ = minecraft_vers.pop();
            download(&cl, &minecraft_vers, L_MINECRAFT_VER, R_MINECRAFT_VER, 1)?;
            minecraft_vers.push(L_MINECRAFT_VER);
        }

        let json = read_to_string(&minecraft_vers).context(anyhow!(
            "Failed to read version manifest from: {minecraft_vers:#?}",
        ))?;

        Ok(from_str(&json)?)
    }
}
