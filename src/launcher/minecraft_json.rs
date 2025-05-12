use std::error::Error;
use std::fs::OpenOptions;
use std::path::PathBuf;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Deserializer;
use tokio::fs::DirBuilder as TkDirBuilder;
use tokio::fs::OpenOptions as TkOpenOptions;
use tokio::io::AsyncWriteExt as TkAsyncWriteExt;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftJson {
    pub libraries: Vec<MinecraftLibs>,
    pub id: String, // Version i.e. 1.21.5
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    pub minimum_launcher_version: usize,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    #[serde(rename = "type")]
    pub reltype: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JavaVersion {
    component: String,
    #[serde(rename = "majorVersion")]
    major_version: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftLibs {
    pub downloads: MinecraftLibDownload,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftLibDownload {
    pub artifact: MinecraftArtifact,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinecraftArtifact {
    pub path: String,
    pub sha1: String,
    pub size: usize,
    pub url: String,
}

impl MinecraftJson {
    /// ```
    /// cache_dir: Path - from the output of crate::launcher::Minecraft download()
    /// ```
    pub fn new(mut cache_dir: PathBuf) -> Result<Self, Box<dyn Error>> {
        cache_dir.push("client.json");
        let mjf = OpenOptions::new().read(true).open(cache_dir)?;
        let mj = Self::deserialize(&mut Deserializer::from_reader(mjf))?;
        Ok(mj)
    }

    /// ```
    /// cl: async Client - (cloning is fine, it's an Arc internally)
    /// root: Path - ~/.local/share/breadlauncher/cache/{version}
    /// I expect the dir struct to be like this
    /// appdir/cache/version/{libraries/, client.jar, client.json}
    /// and the libraries part is where download_libs goes brrr with async
    /// ```
    pub async fn download_libs(&self, cl: Client, root: PathBuf) -> Result<(), Box<dyn Error>> {
        let mut db = TkDirBuilder::new();
        let mut handles = vec![];
        db.recursive(true);

        for l in &self.libraries {
            let mut root2 = root.clone();
            root2.push("libraries");
            let path = l
                .downloads
                .artifact
                .path
                .split("/")
                .into_iter()
                .collect::<Vec<&str>>();

            let last = path.last().unwrap();
            for i in 0..path.len() - 1 {
                root2.push(path.get(i).unwrap());
            }

            let _ = db.create(&root2).await;
            root2.push(last);
            let url = l.downloads.artifact.url.clone();
            let name = l.name.clone();
            let cl2 = cl.clone();
            let j: JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> =
                tokio::spawn(async move {
                    println!("Now requesting: {name}");
                    let res = cl2.get(url).send().await?;
                    let body = res.text().await?;
                    let mut file = TkOpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(root2)
                        .await?;

                    file.write_all(body.as_bytes()).await?;
                    file.sync_all().await?;
                    Ok(())
                });

            handles.push(j);
        }

        for handle in handles {
            let _ = handle.await?;
        }

        Ok(())
    }
}
