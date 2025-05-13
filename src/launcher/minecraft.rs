use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Deserializer;

use crate::utils;

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftVersions {
    pub release: HashMap<String, MinecraftClient>,
    pub snapshot: HashMap<String, MinecraftClient>,
    pub april: HashMap<String, MinecraftClient>,
    pub beta: HashMap<String, MinecraftClient>,
    pub alpha: HashMap<String, MinecraftClient>,
}

impl MinecraftVersions {
    pub fn new(versions: &'static [u8]) -> Result<Self, Box<dyn Error>> {
        let mut de = Deserializer::from_slice(versions);
        let s = Self::deserialize(&mut de)?;
        Ok(s)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftClient {
    client_jar: Arc<str>,
    client_json: Arc<str>,
    client_obfuscation: Arc<str>,
    server_jar: Arc<str>,
    server_obfuscation: Arc<str>,
    version: Arc<str>,
}

impl MinecraftClient {
    /// ```
    /// cl: Client - The reqest async Client (cloning is fine, it's an Arc internally)
    /// approot: Path - ~/.local/share/breadlauncher or %APPDATA%\breadlauncher
    ///
    /// Returns the cache dir for crate::launcher::minecraft_json to use for
    /// the location of downloading it's libs
    /// ```
    pub async fn download(
        &self,
        client: Client,
        approot: impl AsRef<Path>,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let mut p = approot.as_ref().join("cache");
        p.push(self.version.as_ref());
        let cjar = utils::download(&client, &p, "client.jar", &self.client_jar);
        let cjson = utils::download(&client, &p, "client.json", &self.client_json);
        let cobf = utils::download(&client, &p, "client.txt", &self.client_obfuscation);
        let sjar = utils::download(&client, &p, "server.jar", &self.server_jar);
        let sobf = utils::download(&client, &p, "server.txt", &self.server_obfuscation);

        let _ = cjar.await.await?;
        let _ = cjson.await.await?;
        let _ = cobf.await.await?;
        let _ = sjar.await.await?;
        let _ = sobf.await.await?;
        Ok(p)
    }
}
