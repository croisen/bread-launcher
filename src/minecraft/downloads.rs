use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;

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
    pub async fn download(&self, cl: &Client, cache_path: impl AsRef<Path>) -> Result<()> {
        let clju = &self.client.url.clone();
        let cljs = &self.client.sha1.clone();
        let clj =
            utils::download::download_with_sha(cl, &cache_path, "client.jar", clju, cljs, true, 1);

        let slju = &self.server.url.clone();
        let sljs = &self.server.sha1.clone();
        let slj =
            utils::download::download_with_sha(cl, &cache_path, "server.jar", slju, sljs, true, 1);

        clj.await?;
        slj.await?;

        if let Some(clm) = &self.client_mappings {
            let clmu = &clm.url.clone();
            let clms = &clm.sha1.clone();
            utils::download::download_with_sha(cl, &cache_path, "client.txt", clmu, clms, true, 1)
                .await?;
        }

        if let Some(sem) = &self.server_mappings {
            let semu = &sem.url.clone();
            let sems = &sem.sha1.clone();
            utils::download::download_with_sha(cl, &cache_path, "server.txt", semu, sems, true, 1)
                .await?;
        }

        if let Some(ws) = &self.windows_server {
            let wsu = &ws.url.clone();
            let wss = &ws.sha1.clone();
            utils::download::download_with_sha(
                cl,
                &cache_path,
                "windows_server.exe",
                wsu,
                wss,
                true,
                1,
            )
            .await?;
        }

        Ok(())
    }
}
