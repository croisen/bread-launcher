use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_reader};

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

    pub async fn download_asset_json(
        &self,
        cl: &Client,
        cache_dir: impl AsRef<Path>,
    ) -> Result<Value> {
        let mut p = cache_dir.as_ref().join(".minecraft");
        p.push("assets");
        p.push("indexes");
        utils::download::download_with_sha(
            cl,
            &p,
            format!("{}.json", self.id.as_ref()),
            &self.url,
            &self.sha1,
            true,
            1,
        )
        .await?;

        p.push(format!("{}.json", self.id.as_ref()));
        let f = File::open(&p).context(format!("Was opening file {p:?}"))?;
        let _ = p.pop();
        let _ = p.pop();
        let j: Value = from_reader(f)?;
        Ok(j)
    }

    pub async fn download_asset_from_hash(
        &self,
        cl: &Client,
        cache_dir: impl AsRef<Path>,
        hash: &str,
        is_legacy: bool,
    ) -> Result<()> {
        let mut p = cache_dir.as_ref().to_path_buf();
        if is_legacy {
            p.push("virtual");
            p.push("legacy");
        } else {
            p.push("objects");
        }

        let fold = String::from(&hash[0..2]);
        p.push(&fold);
        let url = format!("https://resources.download.minecraft.net/{fold}/{hash}");
        utils::download::download_with_sha(cl, &p, hash, url, hash, true, 1).await?;

        Ok(())
    }
}
