use std::fs::File;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, from_reader};

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

#[derive(Copy, Clone)]
enum AssetType {
    /// Pfft
    Historic,
    Legacy,
    Modern,
}

impl MinecraftAsset {
    pub fn get_id(&self) -> Arc<str> {
        self.id.clone()
    }

    pub fn is_legacy(&self) -> bool {
        self.id.as_ref() == "legacy" || self.id.as_ref() == "pre-1.6"
    }

    pub fn download_asset_json(&self, cl: &Client) -> Result<Map<String, Value>> {
        let mut p = get_assetsdir().join("indexes");
        download_with_sha1(
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
        let j = from_reader::<_, Value>(f)?["objects"]
            .as_object()
            .ok_or(anyhow!(
                "Asset index {} didn't have the asset list",
                self.id
            ))?
            .to_owned();

        Ok(j)
    }

    /// Use the iterated resulting value from self::download_asset_json
    pub fn download_asset(&self, cl: &Client, asset: (&String, &Value)) -> Result<()> {
        let mut p = get_assetsdir();
        let name = asset.0;
        let hash = asset.1["hash"].as_str().ok_or(anyhow!(
            "Asset index object {} did not have the hash value",
            self.id
        ))?;

        let fold = String::from(&hash[0..2]);
        let url = format!("https://resources.download.minecraft.net/{fold}/{hash}");
        match self.get_asset_type() {
            AssetType::Historic => {
                p.extend(name.split("/"));
                let filename = p.file_name().unwrap().display().to_string();
                let _ = p.pop();
                download_with_sha1(cl, &p, filename, url, hash, 1)?;
            }
            AssetType::Legacy => {
                p.push("virtual");
                p.push("legacy");
                p.push(fold);
                download_with_sha1(cl, &p, hash, url, hash, 1)?;
            }
            AssetType::Modern => {
                p.push("objects");
                p.push(fold);
                download_with_sha1(cl, &p, hash, url, hash, 1)?;
            }
        }

        Ok(())
    }

    fn get_asset_type(&self) -> AssetType {
        if self.id.as_ref() == "pre-1.6" {
            AssetType::Historic
        } else if self.id.as_ref() == "legacy" {
            AssetType::Legacy
        } else {
            AssetType::Modern
        }
    }
}
