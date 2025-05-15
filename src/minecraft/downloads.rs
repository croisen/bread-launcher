use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

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
    pub async fn download(&self, cache_path: impl AsRef<Path>) -> Result<()> {
        Ok(())
    }
}
