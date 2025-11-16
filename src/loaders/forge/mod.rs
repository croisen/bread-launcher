use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpmc::Receiver as MultiReceiver;
use std::sync::mpsc::Sender as SingleSender;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

mod libraries;
mod version_manifest;

pub use libraries::ForgeLibrary;
pub use version_manifest::{ForgeVersionManifest, download_forge_json};

use crate::account::Account;
use crate::init::{FULLNAME, get_versiondir};
use crate::loaders::minecraft::Minecraft;
use crate::utils::message::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct Forge {
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    #[serde(default)]
    libraries: Vec<ForgeLibrary>,
    #[serde(default, rename = "jarMods")]
    jar_mods: Vec<ForgeLibrary>,

    #[serde(skip)]
    instance_dir: PathBuf,
    #[serde(skip)]
    mc_ver: String,
    #[serde(skip)]
    forge_ver: String,
}

impl Forge {
    pub fn new(
        instance_dir: impl AsRef<Path>,
        mc_ver: impl AsRef<str>,
        forge_ver: impl AsRef<str>,
    ) -> Result<Self> {
        let json = get_versiondir().join(format!("forge-{}.json", forge_ver.as_ref()));
        let json = read_to_string(&json).context(format!("Failed to read {json:?}"))?;
        let mut m: Self = from_str(&json)?;
        m.instance_dir = instance_dir.as_ref().to_path_buf();
        m.mc_ver = mc_ver.as_ref().to_string();
        m.forge_ver = forge_ver.as_ref().to_string();

        Ok(m)
    }

    pub fn download(
        &self,
        cl: Client,
        steps: (Arc<AtomicUsize>, Arc<AtomicUsize>),
        tx: SingleSender<Message>,
        rx: MultiReceiver<()>,
    ) -> Result<()> {
        let m = Minecraft::new(&self.instance_dir, &self.mc_ver)?;
        m.download(cl.clone(), steps.clone(), tx.clone(), rx.clone())?;

        let (steps, total) = steps;
        total.fetch_add(
            self.libraries.len() + self.jar_mods.len(),
            Ordering::Relaxed,
        );
        for lib in &self.libraries {
            if rx.try_recv().is_ok() {
                let _ = tx.send(Message::errored("Stop signal received"));
                return Ok(());
            }

            steps.fetch_add(1, Ordering::Relaxed);
            match lib {
                ForgeLibrary::A(l) => {
                    l.download_library(cl.clone(), &self.instance_dir)?;
                    let _ = tx.send(Message::downloading(format!(
                        "Downloading library: {}",
                        l.name
                    )));
                }
                ForgeLibrary::B(l) => {
                    l.download_library(cl.clone(), &self.instance_dir)?;
                    let _ = tx.send(Message::downloading(format!(
                        "Downloading library: {}",
                        l.name
                    )));
                }
            }
        }

        for lib in &self.jar_mods {
            if rx.try_recv().is_ok() {
                let _ = tx.send(Message::errored("Stop signal received"));
                return Ok(());
            }

            steps.fetch_add(1, Ordering::Relaxed);
            match lib {
                ForgeLibrary::A(l) => {
                    l.download_library(cl.clone(), &self.instance_dir)?;
                    let _ = tx.send(Message::downloading(format!(
                        "Downloading library: {}",
                        l.name
                    )));
                }
                ForgeLibrary::B(l) => {
                    l.download_library(cl.clone(), &self.instance_dir)?;
                    let _ = tx.send(Message::downloading(format!(
                        "Downloading library: {}",
                        l.name
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn run(self, ram: usize, account: Arc<Account>) -> Result<Child> {
        let m = Minecraft::new(&self.instance_dir, &self.mc_ver)?;
        let jre = m.get_jre_path();
        let mut jvm_args = m.get_jvm_args(ram);
        let mut mc_args = if m.asset_index.is_legacy() {
            m.get_mc_args_legacy(account)
        } else {
            m.get_mc_args(account)
        };

        let mut paths = vec![];
        for l in &self.libraries {
            match l {
                ForgeLibrary::A(l) => {
                    if let Some(p) = l.get_path() {
                        paths.push(p)
                    }
                }
                ForgeLibrary::B(l) => paths.push(l.get_path()),
            }
        }

        for l in &self.jar_mods {
            match l {
                ForgeLibrary::A(l) => {
                    if let Some(p) = l.get_path() {
                        paths.push(p)
                    }
                }
                ForgeLibrary::B(l) => paths.push(l.get_path()),
            }
        }

        let libs = paths
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<String>>();

        let classpaths = if cfg!(windows) {
            let mut p = libs
                .iter()
                .filter(|l| !l.split("\\").last().unwrap().contains("natives"))
                .map(|l| l.as_str())
                .collect::<Vec<&str>>()
                .join(";");

            if !p.is_empty() {
                p.insert(0, ';');
            }

            p
        } else {
            let mut p = libs
                .iter()
                .filter(|l| !l.split("/").last().unwrap().contains("natives"))
                .map(|l| l.as_str())
                .collect::<Vec<&str>>()
                .join(":");

            if !p.is_empty() {
                p.insert(0, ':');
            }

            p
        };

        let len = jvm_args.len();
        // Last one is the main class used by mc but it needs to be forge's
        jvm_args[len - 1] = self.main_class.clone();
        // We gotta modify the classpaths to include the libraries used by forge
        jvm_args[len - 2] += &classpaths;

        let len = mc_args.len();
        // Gotta have our own branding in there
        mc_args[len - 1] = format!("forge-{}/{}", self.forge_ver, FULLNAME);

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
