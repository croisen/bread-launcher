use std::fs::create_dir_all;
use std::fs::File;
use std::io;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, Value};

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
    pub async fn download(&self, cl: &Client, cache_dir: impl AsRef<Path>) -> Result<String> {
        utils::download::download_with_sha(
            cl,
            cache_dir.as_ref(),
            "assets.json",
            &self.url.clone(),
            &self.sha1.clone(),
            true,
            1,
        )
        .await?;

        let mut p = cache_dir.as_ref().join("assets.json");
        let f = File::open(&p)?;
        let j: Value = from_reader(f)?;
        let _ = p.pop();
        p.push(".minecraft");
        p.push("assets");
        match &j["objects"].as_object() {
            Some(assets) => {
                for (name, asset) in assets.iter() {
                    let sha1: Arc<str> = Arc::from(asset.get("hash").unwrap().as_str().unwrap());
                    let fold = String::from(&sha1.as_ref()[0..2]);
                    p.push(&fold);
                    let url = Arc::from(format!(
                        "https://resources.download.minecraft.net/{fold}/{sha1}"
                    ));

                    utils::download::download_with_sha(
                        cl,
                        &p,
                        sha1.clone().as_ref(),
                        &url,
                        &sha1.clone(),
                        true,
                        1,
                    )
                    .await?;

                    p.push(sha1.as_ref());
                    let mut s = File::open(&p)?;
                    let _ = p.pop();
                    let _ = p.pop();
                    p.push("virtual");
                    p.push("legacy");
                    p.push(&fold);
                    create_dir_all(&p)?;
                    p.push(sha1.as_ref());

                    if !p.is_file() {
                        let mut d = File::create_new(&p)?;
                        io::copy(&mut s, &mut d)?;
                    }

                    let _ = p.pop();
                    let _ = p.pop();
                    let _ = p.pop();
                    let _ = p.pop();
                }
            }
            None => {
                return Err(anyhow!(
                    "The objects key wasn't there, in the assets json???"
                ));
            }
        }

        Ok(self.id.as_ref().to_string())
    }
}
