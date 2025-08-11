use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_slice};
use tokio::fs::read;

use crate::init::get_assetsdir;
use crate::utils::download::download_with_sha1;

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

    pub fn hashes(&self, v: &Value) -> Result<Vec<String>> {
        let hashes = v["objects"]
            .as_object()
            .ok_or(anyhow!("Asset index didn't containt an objects value?"))
            .context(format!("Array index was: {}", self.id))?
            .values()
            .collect::<Vec<&Value>>()
            .iter()
            .map(|vv| vv["hash"].as_str().unwrap().to_string())
            .collect::<Vec<String>>();

        Ok(hashes)
    }

    pub fn is_legacy(&self, v: &Value) -> bool {
        v["virtual"].as_bool().unwrap_or(false)
    }

    pub async fn download_and_parse_asset_json(&self, cl: Client) -> Result<Value> {
        let p = get_assetsdir().join("indexes");
        download_with_sha1(
            cl,
            &p,
            format!("{}.json", self.id.as_ref()),
            &self.url,
            &self.sha1,
            1,
        )
        .await?;

        let j = self.parse_assets_file().await?;

        Ok(j)
    }

    // Gotta get the objects array first from the result of download_asset_json
    // and use the contents of that here one by one
    pub async fn download_asset_from_json(
        &self,
        cl: Client,
        hash: &str,
        is_legacy: bool,
    ) -> Result<()> {
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
        download_with_sha1(cl, &p, hash, url, hash, 1).await?;

        Ok(())
    }

    pub async fn parse_assets_file(&self) -> Result<Value> {
        let mut p = get_assetsdir().join("indexes");
        p.push(format!("{}.json", self.id.as_ref()));
        let f = read(&p).await.context(format!("Was opening file {p:?}"))?;
        let assets: Value = from_slice(f.as_slice())?;

        Ok(assets)
    }
}
