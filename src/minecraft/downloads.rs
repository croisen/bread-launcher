use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftMidDownload {
    pub sha1: Arc<str>,
    pub size: usize,
    pub url: Arc<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftDownload {
    pub client: MinecraftMidDownload,
    pub client_mappings: Option<MinecraftMidDownload>,
    pub server: MinecraftMidDownload,
    pub server_mappings: Option<MinecraftMidDownload>,
    pub windows_server: Option<MinecraftMidDownload>,
}

impl MinecraftDownload {
    pub fn download_client(
        &self,
        cl: &Client,
        name: impl AsRef<str> + Send + Sync,
        cache_path: impl AsRef<Path> + Send + Sync,
    ) -> Result<()> {
        utils::download::download_with_sha(
            cl,
            cache_path,
            name,
            &self.client.url,
            &self.client.sha1,
            1,
        )?;

        Ok(())
    }
}
