use std::fs::read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpmc::Sender;
use std::time::SystemTime;

use anyhow::{Context, Result, anyhow, bail};
use rand::{RngCore, rng};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use uuid::Builder as UB;
use uuid::Version;

mod arguments;
mod assets;
mod downloads;
mod java_version;
mod libraries;
mod organized;
mod rules;
mod version_manifest;

pub use arguments::MinecraftArgument;
pub use assets::MinecraftAsset;
pub use downloads::MinecraftDownload;
pub use java_version::MinecraftJavaVersion;
pub use libraries::MinecraftLibrary;
pub use organized::MVOrganized;
pub use rules::MinecraftRule;
pub use version_manifest::MinecraftVersionManifest;

use crate::account::Account;
use crate::utils::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minecraft {
    arguments: Option<Arc<MinecraftArgument>>,
    #[serde(rename = "minecraftArguments")]
    minecraft_arguments: Option<Arc<arguments::Argument>>,
    #[serde(rename = "assetIndex")]
    asset_index: Arc<MinecraftAsset>,
    downloads: Arc<MinecraftDownload>,
    #[serde(default, rename = "javaVersion")]
    java_version: Arc<MinecraftJavaVersion>,
    libraries: Vec<Arc<MinecraftLibrary>>,

    id: Arc<str>,
    #[serde(rename = "mainClass")]
    main_class: Arc<str>,
    #[serde(rename = "minimumLauncherVersion")]
    minimum_launcher_version: usize,
    #[serde(rename = "releaseTime")]
    release_time: Arc<str>,
    time: Arc<str>,
    #[serde(rename = "type")]
    release_type: Arc<str>,

    #[serde(skip_deserializing)]
    appdir: Arc<PathBuf>,
    #[serde(skip_deserializing)]
    instance_dir: Arc<PathBuf>,
}

impl Minecraft {
    pub fn new(instance_dir: impl AsRef<Path>, version: impl AsRef<str>) -> Result<Self> {
        let mut ad = instance_dir.as_ref().to_path_buf();
        let _ = ad.pop();
        let _ = ad.pop();
        ad.push("minecraft_cache");
        ad.push("versions");

        let json = ad.join(format!("{}.json", version.as_ref()));
        let f = read(&json).context(format!("Failed to read {json:?}"))?;
        let mut de = Deserializer::from_slice(f.as_ref());
        let mut m = Self::deserialize(&mut de).context(format!(
            "Failed to desrialize minecraft client data from {json:?}"
        ))?;

        let _ = ad.pop();
        let _ = ad.pop();
        m.appdir = Arc::new(ad);
        m.instance_dir = Arc::new(instance_dir.as_ref().to_path_buf());

        Ok(m)
    }

    pub fn get_jre_path(&self) -> Result<String> {
        let mut jre = self.appdir.join("java");
        jre.push(format!("{:0>2}", self.java_version.get_version()));
        jre.push("bin");

        #[cfg(target_family = "unix")]
        jre.push("java");
        #[cfg(target_family = "windows")]
        jre.push("javaw.exe");

        let res = jre
            .to_str()
            .ok_or(
                anyhow!("Can't convert path to a valid UTF-8 string?")
                    .context(format!("Path: {jre:?}")),
            )?
            .to_owned();

        Ok(res)
    }

    /// ram in MB
    pub fn get_jvm_args(&self, ram: usize) -> Vec<String> {
        let mut dir = self.appdir.join("minecraft_cache");
        let natives = self
            .instance_dir
            .join("natives")
            .to_str()
            .unwrap()
            .to_string();
        let mut libs = self
            .libraries
            .iter()
            .map(|l| l.get_path(&dir))
            .filter(|p| p.is_some())
            .map(|p| p.as_ref().unwrap().to_str().unwrap().to_string())
            .collect::<Vec<String>>();

        dir.push("versions");
        dir.push(format!("{}.jar", self.id.as_ref()));
        libs.push(dir.to_str().unwrap().to_string());

        #[cfg(target_family = "unix")]
        let classpaths = libs
            .iter()
            .filter(|l| !l.split("/").last().unwrap().contains("natives"))
            .map(|l| l.as_str())
            .collect::<Vec<&str>>()
            .join(":");
        #[cfg(target_family = "windows")]
        let classpaths = libs
            .iter()
            .filter(|l| !l.split("\\").last().unwrap().contains("natives"))
            .map(|l| l.as_str())
            .collect::<Vec<&str>>()
            .join(";");

        vec![
            format!("-Xms{ram}M"),
            format!("-Xmx{ram}M"),
            "-Xss1M".to_string(),
            "-Dminecraft.launcher.brand=bread-launcher".to_string(),
            format!("-Dminecraft.launcher.version={}", env!("CARGO_PKG_VERSION")),
            format!("-Djava.library.path={natives}"),
            "-cp".to_string(),
            classpaths,
            self.main_class.as_ref().to_string(),
        ]
    }

    pub fn get_mc_args_legacy(&self, account: Arc<Account>) -> Result<Vec<String>> {
        let mut dir = self.appdir.join("minecraft_cache");
        dir.push("assets");
        let assets = dir
            .to_str()
            .ok_or(anyhow!("Path is not valid unicode???"))?
            .to_string();
        let game_dir = self
            .instance_dir
            .to_str()
            .ok_or(anyhow!("Path is not valid unicode???"))?
            .to_string();

        let mc = vec![
            "--assetIndex".to_string(),
            self.asset_index.get_id().to_string(),
            "--gameDir".to_string(),
            game_dir,
            "--assetsDir".to_string(),
            assets,
            "--username".to_string(),
            account.name.to_string(),
            "--userProperties".to_string(),
            /* TODO user_properties, */ "{}".to_string(),
            "--uuid".to_string(),
            account.uuid.to_string(),
            "--accessToken".to_string(),
            account.token.to_string(),
            "--version".to_string(),
            self.id.as_ref().to_string(),
            "--versionType".to_string(),
            format!("bread-launcher-{}", env!("CARGO_PKG_VERSION")),
        ];

        Ok(mc)
    }

    pub fn get_mc_args(&self, account: Arc<Account>) -> Result<Vec<String>> {
        let mut dir = self.appdir.join("minecraft_cache");
        dir.push("assets");
        let assets = dir
            .to_str()
            .ok_or(anyhow!("Path is not valid unicode???"))?
            .to_string();
        let game_dir = self
            .instance_dir
            .to_str()
            .ok_or(anyhow!("Path is not valid unicode???"))?
            .to_string();

        let mc = vec![
            "--assetIndex".to_string(),
            self.asset_index.get_id().to_string(),
            "--gameDir".to_string(),
            game_dir,
            "--assetsDir".to_string(),
            assets,
            "--username".to_string(),
            account.name.to_string(),
            "--userType".to_string(),
            account.account_type.to_string(),
            "--userProperties".to_string(),
            /* TODO user_properties, */ "{}".to_string(),
            "--uuid".to_string(),
            account.uuid.to_string(),
            "--accessToken".to_string(),
            account.token.to_string(),
            "--version".to_string(),
            self.id.as_ref().to_string(),
            "--versionType".to_string(),
            format!("bread-launcher-{}", env!("CARGO_PKG_VERSION")),
        ];

        Ok(mc)
    }

    pub fn new_instance(&self) -> Result<Self> {
        log::info!("Creating new instance for MC ver {}", self.id.as_ref());
        let mut s = self.clone();
        let mut c = self.appdir.join("instances");
        let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let mut rb: [u8; 10] = [0; 10];
        rng().fill_bytes(&mut rb);

        let u = UB::from_unix_timestamp_millis(ts.as_millis().try_into()?, &rb)
            .with_version(Version::SortRand)
            .into_uuid()
            .to_string();

        c.push(&u);
        s.instance_dir = Arc::new(c);
        log::info!(
            "New instance created in dir {:?} with MC ver {}",
            s.instance_dir,
            s.id.as_ref()
        );

        Ok(s)
    }

    pub fn get_cache_dir(&self) -> Arc<PathBuf> {
        self.instance_dir.clone()
    }

    pub fn download_jre(
        &self,
        cl: Client,
        step: Arc<AtomicUsize>,
        total_steps: Arc<AtomicUsize>,
        tx: Sender<Message>,
    ) -> Result<()> {
        total_steps.store(2, Ordering::Relaxed);
        step.store(1, Ordering::Relaxed);
        let _ = tx.send(Message::Downloading(format!(
            "Downloading JRE version {:0>2}",
            self.java_version.get_version()
        )));

        self.java_version.download(&cl, self.appdir.as_ref())?;
        step.fetch_add(1, Ordering::Relaxed);
        let _ = tx.send(Message::Message("JRE Extraction finished".to_string()));

        Ok(())
    }

    pub fn download_client(
        &self,
        cl: Client,
        step: Arc<AtomicUsize>,
        total_steps: Arc<AtomicUsize>,
        tx: Sender<Message>,
    ) -> Result<()> {
        let dir = self.appdir.join("minecraft_cache");
        let client = format!("{}.jar", self.id.as_ref());
        let client_dir = dir.join("versions");

        total_steps.store(self.libraries.len() + 1, Ordering::Relaxed);
        step.store(1, Ordering::Relaxed);
        let _ = tx.send(Message::Downloading("Downloading client jar".to_string()));
        self.downloads.download_client(&cl, client, client_dir)?;
        for lib in &self.libraries {
            let path = lib.get_path(&dir);
            step.fetch_add(1, Ordering::Relaxed);
            if path.is_none() {
                continue;
            }

            let path = path.unwrap();
            let path_str = path
                .to_str()
                .ok_or(
                    anyhow!("Cannot convert path to valid UTF-8?")
                        .context(format!("Path: {path:?}")),
                )?
                .strip_prefix(
                    dir.to_str().ok_or(
                        anyhow!("Cannot convert path to valid UTF-8?")
                            .context(format!("Path: {dir:?}")),
                    )?,
                )
                .unwrap();

            let _ = tx.send(Message::Downloading(format!("Downloading lib: {path_str}")));
            lib.download_library(&cl, &dir, self.instance_dir.as_ref())?;
        }

        Ok(())
    }

    pub fn download_assets(
        &self,
        cl: Client,
        step: Arc<AtomicUsize>,
        total_steps: Arc<AtomicUsize>,
        tx: Sender<Message>,
    ) -> Result<()> {
        let dir = self.appdir.join("minecraft_cache");
        let assets_dir = dir.join("assets");
        total_steps.store(1, Ordering::Relaxed);
        step.store(1, Ordering::Relaxed);
        let _ = tx.send(Message::Downloading("Downloading asset index".to_string()));

        let v = self.asset_index.download_asset_json(&cl, &assets_dir)?;
        let is_legacy = v["virtual"].as_bool().unwrap_or(false);
        let assets = v["objects"]
            .as_object()
            .ok_or(anyhow!("Asset index didn't containt an objects value?"))
            .context(format!("Array index was: {}", self.asset_index.get_id()))?;

        total_steps.fetch_add(assets.len(), Ordering::Relaxed);
        let _ = tx.send(Message::Downloading("Downloading assets".to_string()));
        for (_, asset) in assets {
            let hash = asset["hash"]
                .as_str()
                .ok_or(anyhow!("Asset hash doesn't exist?"))
                .context(format!("Array index was: {}", self.asset_index.get_id()))?;

            step.fetch_add(1, Ordering::Relaxed);
            self.asset_index
                .download_asset_from_json(&cl, &assets_dir, hash, is_legacy)?;
        }

        Ok(())
    }

    pub fn run(&self, cl: Client, ram: usize, account: Arc<Account>) -> Result<()> {
        let mut assets_dir = self.appdir.join("minecraft_cache");
        assets_dir.push("assets");
        let is_legacy = self.asset_index.download_asset_json(&cl, &assets_dir)?["virtual"]
            .as_bool()
            .unwrap_or(false);

        let jre = self.get_jre_path()?;
        let jvm_args = self.get_jvm_args(ram);
        let mc_args = if is_legacy {
            self.get_mc_args_legacy(account)?
        } else {
            self.get_mc_args(account)?
        };

        let mut child = Command::new(&jre)
            .current_dir(self.instance_dir.as_ref())
            .args(&jvm_args)
            .args(&mc_args)
            .spawn()
            .context(format!(
                "Failed to start minecraft with jvm {jre}\njvm_args: {jvm_args:#?}"
            ))?;

        let status = child.wait()?;
        if let Some(status) = status.code() {
            log::info!("Run exit status: {status}");
            if status != 0 {
                log::error!("jvm: {jre}");
                log::error!("jvm: {jvm_args:#?}");
                log::error!("jvm: {mc_args:#?}");
                bail!("Java's exit status is not successfull ({status})");
            }
        }

        Ok(())
    }
}
