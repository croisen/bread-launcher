use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpmc::Receiver as MultiReceiver;
use std::sync::mpsc::Sender as SingleSender;
use std::time::SystemTime;

use anyhow::{Context, Result};
use rand::{RngCore, rng};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use uuid::Builder as UB;

mod arguments;
mod assets;
mod downloads;
mod java_version;
mod libraries;
mod organized;
mod rules;
mod version_manifest;

pub use arguments::{Argument, MinecraftArgument};
pub use assets::MinecraftAsset;
pub use downloads::MinecraftDownload;
pub use java_version::MinecraftJavaVersion;
pub use libraries::MinecraftLibrary;
pub use organized::MVOrganized;
pub use rules::MinecraftRule;
pub use version_manifest::{MinecraftVersion, MinecraftVersionManifest};

use crate::account::Account;
use crate::init::{
    FULLNAME, VERSION, get_assetsdir, get_cachedir, get_instancedir, get_javadir, get_versiondir,
};
use crate::utils::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minecraft {
    arguments: Option<MinecraftArgument>,
    #[serde(rename = "minecraftArguments")]
    minecraft_arguments: Option<Argument>,
    #[serde(rename = "assetIndex")]
    asset_index: MinecraftAsset,
    downloads: MinecraftDownload,
    #[serde(default, rename = "javaVersion")]
    java_version: MinecraftJavaVersion,
    pub libraries: Vec<MinecraftLibrary>,

    id: String,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    minimum_launcher_version: usize,
    #[serde(rename = "releaseTime")]
    release_time: String,
    time: String,
    #[serde(rename = "type")]
    release_type: String,

    #[serde(skip)]
    instance_dir: PathBuf,
}

impl Minecraft {
    pub fn new(instance_dir: impl AsRef<Path>, version: impl AsRef<str>) -> Result<Self> {
        let json = get_versiondir().join(format!("{}.json", version.as_ref()));
        let json = read_to_string(&json).context(format!("Failed to read {json:?}"))?;
        let mut m: Self = from_str(&json)?;
        m.instance_dir = instance_dir.as_ref().to_path_buf();

        Ok(m)
    }

    pub fn get_jre_path(&self) -> String {
        let mut jre = get_javadir();
        jre.push(format!("{:0>2}", self.java_version.get_version()));
        jre.push("bin");

        if cfg!(windows) {
            jre.push("javaw.exe");
        } else {
            jre.push("java");
        }

        jre.display().to_string()
    }

    /// ram in MB
    pub fn get_jvm_args(&self, ram: usize) -> Vec<String> {
        let natives = self.instance_dir.join("natives").display().to_string();
        let mut libs = self
            .libraries
            .iter()
            .map(|l| l.get_path())
            .filter(|p| p.is_some())
            .map(|p| p.as_ref().unwrap().display().to_string())
            .collect::<Vec<String>>();

        let client = get_versiondir().join(format!("{}.jar", self.id));
        libs.push(client.display().to_string());

        let classpaths = if cfg!(windows) {
            libs.iter()
                .filter(|l| !l.split("\\").last().unwrap().contains("natives"))
                .map(|l| l.as_str())
                .collect::<Vec<&str>>()
                .join(";")
        } else {
            libs.iter()
                .filter(|l| !l.split("/").last().unwrap().contains("natives"))
                .map(|l| l.as_str())
                .collect::<Vec<&str>>()
                .join(":")
        };

        let jvm_args = vec![
            format!("-Xms{ram}M"),
            format!("-Xmx{ram}M"),
            "-Xss1M".to_string(),
            "-Dminecraft.launcher.brand=bread-launcher".to_string(),
            format!("-Dminecraft.launcher.version={}", VERSION),
            format!("-Djava.library.path={natives}"),
            "-cp".to_string(),
            classpaths,
            self.main_class.clone(),
        ];

        jvm_args
    }

    pub fn get_mc_args_legacy(&self, account: Arc<Account>) -> Vec<String> {
        let assets = get_assetsdir().display().to_string();
        let game_dir = self.instance_dir.join(".minecraft").display().to_string();

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
            self.id.clone(),
            "--versionType".to_string(),
            FULLNAME.to_string(),
        ];

        mc
    }

    pub fn get_mc_args(&self, account: Arc<Account>) -> Vec<String> {
        let assets = get_assetsdir().display().to_string();
        let game_dir = self.instance_dir.join(".minecraft").display().to_string();

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
            account.account_type.to_string().to_ascii_lowercase(),
            "--userProperties".to_string(),
            /* TODO user_properties, */ "{}".to_string(),
            "--uuid".to_string(),
            account.uuid.to_string(),
            "--accessToken".to_string(),
            account.token.to_string(),
            "--version".to_string(),
            self.id.to_string(),
            "--versionType".to_string(),
            FULLNAME.into(),
        ];

        mc
    }

    pub fn new_instance(&self) -> Result<Self> {
        log::info!("Creating new instance for MC ver {}", self.id);
        let mut s = self.clone();
        let mut c = get_instancedir();
        let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let mut rb: [u8; 10] = [0; 10];
        rng().fill_bytes(&mut rb);

        let u = UB::from_unix_timestamp_millis(ts.as_millis().try_into()?, &rb)
            .into_uuid()
            .to_string();

        c.push(&u);
        s.instance_dir = c;
        log::info!(
            "New instance created in dir {:?} with MC ver {}",
            s.instance_dir,
            s.id
        );

        Ok(s)
    }

    pub fn get_cache_dir(&self) -> &Path {
        self.instance_dir.as_ref()
    }

    pub fn download_jre(
        &self,
        cl: Client,
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        rx: MultiReceiver<()>,
    ) -> Result<bool> {
        let (step, total_steps) = steps;
        total_steps.store(2, Ordering::Relaxed);
        step.store(1, Ordering::Relaxed);
        if rx.try_recv().is_ok() {
            let _ = tx.send(Message::errored("Stop signal received"));
            return Ok(false);
        }

        let _ = tx.send(Message::downloading(format!(
            "Downloading JRE version {:0>2}",
            self.java_version.get_version()
        )));

        if rx.try_recv().is_ok() {
            let _ = tx.send(Message::errored("Stop signal received"));
            return Ok(false);
        }

        self.java_version.download(&cl)?;
        step.fetch_add(1, Ordering::Relaxed);
        let _ = tx.send(Message::msg("JRE Extraction finished"));
        if rx.try_recv().is_ok() {
            let _ = tx.send(Message::errored("Stop signal received"));
            return Ok(false);
        }

        Ok(true)
    }

    pub fn download_client(
        &self,
        cl: Client,
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        rx: MultiReceiver<()>,
    ) -> Result<bool> {
        let (step, total_steps) = steps;
        let client = format!("{}.jar", self.id);
        total_steps.store(self.libraries.len() + 1, Ordering::Relaxed);
        step.store(1, Ordering::Relaxed);
        let _ = tx.send(Message::downloading("Downloading client jar"));
        if rx.try_recv().is_ok() {
            let _ = tx.send(Message::errored("Stop signal received"));
            return Ok(false);
        }

        self.downloads.download_client(&cl, client)?;
        if rx.try_recv().is_ok() {
            let _ = tx.send(Message::errored("Stop signal received"));
            return Ok(false);
        }

        let dir = get_cachedir();
        for lib in &self.libraries {
            if rx.try_recv().is_ok() {
                let _ = tx.send(Message::errored("Stop signal received"));
                return Ok(false);
            }

            let path = lib.get_path();
            step.fetch_add(1, Ordering::Relaxed);
            if path.is_none() {
                continue;
            }

            let path = path.unwrap();
            let path_str = path.strip_prefix(&dir)?.display().to_string();
            let _ = tx.send(Message::downloading(format!("Downloading lib: {path_str}")));
            lib.download_library(cl.clone(), &self.instance_dir)?;
        }

        Ok(true)
    }

    pub fn download_assets(
        &self,
        cl: Client,
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        rx: MultiReceiver<()>,
    ) -> Result<bool> {
        let (step, total_steps) = steps;
        total_steps.store(1, Ordering::Relaxed);
        step.store(1, Ordering::Relaxed);
        let _ = tx.send(Message::downloading("Downloading asset index"));

        let v = self.asset_index.download_asset_json(&cl)?;
        total_steps.fetch_add(v.len(), Ordering::Relaxed);
        let _ = tx.send(Message::downloading("Downloading assets"));
        if rx.try_recv().is_ok() {
            let _ = tx.send(Message::errored("Stop signal received"));
            return Ok(false);
        }

        let v = self.asset_index.download_asset_json(&cl)?;
        if rx.try_recv().is_ok() {
            let _ = tx.send(Message::errored("Stop signal received"));
            return Ok(false);
        }

        total_steps.fetch_add(v.len(), Ordering::Relaxed);
        let _ = tx.send(Message::downloading("Downloading assets"));
        for asset in &v {
            if rx.try_recv().is_ok() {
                let _ = tx.send(Message::errored("Stop signal received"));
                return Ok(false);
            }

            step.fetch_add(1, Ordering::Relaxed);
            self.asset_index.download_asset(&cl, asset)?;
        }

        Ok(true)
    }

    pub fn run(self, ram: usize, account: Arc<Account>) -> Result<Child> {
        let jre = self.get_jre_path();
        let jvm_args = self.get_jvm_args(ram);
        let mc_args = if self.asset_index.is_legacy() {
            self.get_mc_args_legacy(account)
        } else {
            self.get_mc_args(account)
        };

        let child = Command::new(&jre)
            .current_dir(&self.instance_dir)
            .args(&jvm_args)
            .args(&mc_args)
            .spawn()
            .context(format!(
                "Failed to start minecraft with jvm {jre}\njvm_args: {jvm_args:#?}"
            ))?;

        Ok(child)
    }
}
