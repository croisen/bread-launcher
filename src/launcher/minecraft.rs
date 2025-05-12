use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

use reqwest::Client;
use serde::Deserialize;
use tokio::fs::DirBuilder as TkDirBuilder;
use tokio::fs::OpenOptions as TkOpenOptions;
use tokio::io::AsyncWriteExt as TkAsyncWriteExt;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftVersions {
    pub release: HashMap<String, Minecraft>,
    pub snapshot: HashMap<String, Minecraft>,
    pub april: HashMap<String, Minecraft>,
    pub beta: HashMap<String, Minecraft>,
    pub alpha: HashMap<String, Minecraft>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Minecraft {
    client_jar: String,
    client_json: String,
    version: String,
}

impl Minecraft {
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
        mut approot: PathBuf,
    ) -> Result<PathBuf, Box<dyn Error>> {
        approot.push("cache");
        approot.push(&self.version);
        TkDirBuilder::new().recursive(true).create(&approot).await?;

        approot.push("client.jar");
        let jarc = client.clone();
        let jars = self.client_jar.clone();
        let mut jarf = TkOpenOptions::new()
            .write(true)
            .create(true)
            .open(&approot)
            .await?;
        let _ = approot.pop();
        approot.push("client.json");
        let jsoc = client.clone();
        let jsos = self.client_json.clone();
        let mut jsof = TkOpenOptions::new()
            .write(true)
            .create(true)
            .open(&approot)
            .await?;

        let jar: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> = tokio::spawn(async move {
            let res = jarc.get(jars).send().await?;
            let body = res.text().await?;
            jarf.write_all(body.as_bytes()).await?;
            jarf.sync_all().await?;
            Ok(())
        });

        let jso: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> = tokio::spawn(async move {
            let res = jsoc.get(jsos).send().await?;
            let body = res.text().await?;
            jsof.write_all(body.as_bytes()).await?;
            jsof.sync_all().await?;
            Ok(())
        });

        let _ = jar.await?;
        let _ = jso.await?;
        let _ = approot.pop();
        Ok(approot)
    }
}
