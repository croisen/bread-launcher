use std::fs::File;
use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_reader};

use crate::init::get_assetsdir;
use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftAsset {
    id: Arc<str>,
    sha1: Arc<str>,
    size: usize,
    #[serde(rename = "totalSize")]
    total_size: usize,
    url: Arc<str>,
}

impl MinecraftAsset {
    pub fn get_id(&self) -> Arc<str> {
        self.id.clone()
    }

    pub fn download_asset_json(&self, cl: &Client) -> Result<Value> {
        let mut p = get_assetsdir().join("indexes");
        utils::download::download_with_sha(
            cl,
            &p,
            format!("{}.json", self.id.as_ref()),
            &self.url,
            &self.sha1,
            1,
        )?;

        p.push(format!("{}.json", self.id.as_ref()));
        let f = File::open(&p).context(format!("Was opening file {p:?}"))?;
        let _ = p.pop();
        let _ = p.pop();
        let j: Value = from_reader(f)?;
        Ok(j)
    }

    // Gotta get the objects array first from the result of download_asset_json
    // and use the contents of that here one by one
    pub fn download_asset_from_json(&self, cl: &Client, hash: &str, is_legacy: bool) -> Result<()> {
        let mut p = get_assetsdir();
        if is_legacy {
            p.push("virtual");
            p.push("legacy");
        } else {
            p.push("objects");
        }

        let fold = String::from(&hash[0..2]);
        p.push(&fold);
        let url = format!("https://resources.download.minecraft.net/{fold}/{hash}");
        utils::download::download_with_sha(cl, &p, hash, url, hash, 1)?;

        Ok(())
    }
}
