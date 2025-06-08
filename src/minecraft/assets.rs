use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
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
        let mut p = cache_dir.as_ref().join(".minecraft");
        p.push("assets");
        p.push("indexes");
        utils::download::download_with_sha(
            cl,
            &p,
            &format!("{}.json", self.id.as_ref()),
            &self.url.clone(),
            &self.sha1.clone(),
            true,
            1,
        )
        .await?;

        p.push(format!("{}.json", self.id.as_ref()));
        let f = File::open(&p).context(format!("Was opening file {p:?}"))?;
        let _ = p.pop();
        let _ = p.pop();
        let j: Value = from_reader(f)?;
        if j["virtual"].as_bool().unwrap_or(false) {
            p.push("virtual");
            p.push("legacy");
        } else {
            p.push("objects");
        }

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
