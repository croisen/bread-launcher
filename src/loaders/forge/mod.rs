use std::fs::{create_dir_all, read_to_string};
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
use crate::init::{FULLNAME, get_libdir, get_vanilla_path, get_versiondir};
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
    #[serde(default, rename = "mavenFiles")]
    maven_files: Vec<ForgeLibrary>,

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
            self.libraries.len() + self.jar_mods.len() + self.maven_files.len(),
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

        for lib in &self.maven_files {
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
        // Surprisingly jopt will not parse the game directory if it
        // does not exist
        let game_dir = self.instance_dir.join(".minecraft");
        if !game_dir.exists() {
            create_dir_all(game_dir)?;
        }

        let m = Minecraft::new(&self.instance_dir, &self.mc_ver)?;
        let jre = m.get_jre_path();
        let mut jvm_args = m.get_jvm_args(ram);
        let mut mc_args = if m.asset_index.is_legacy() {
            m.get_mc_args_legacy(account)
        } else {
            m.get_mc_args(account)
        };

        let mut paths = vec![];
        let mut mpaths = vec![];
        let mut client = get_vanilla_path(&self.mc_ver);
        client.set_extension("jar");
        paths.push(client);
        for l in &m.libraries {
            if let Some(p) = l.get_path() {
                paths.push(p);
            }
        }

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

        for l in &self.maven_files {
            match l {
                ForgeLibrary::A(l) => {
                    if let Some(p) = l.get_path() {
                        mpaths.push(p)
                    }
                }
                ForgeLibrary::B(l) => mpaths.push(l.get_path()),
            }
        }

        let classpaths = if cfg!(windows) {
            paths
                .iter()
                .filter(|l| {
                    !l.file_name()
                        .unwrap()
                        .display()
                        .to_string()
                        .contains("natives")
                })
                .map(|l| l.display().to_string())
                .collect::<Vec<String>>()
                .join(";")
        } else {
            paths
                .iter()
                .filter(|l| {
                    !l.file_name()
                        .unwrap()
                        .display()
                        .to_string()
                        .contains("natives")
                })
                .map(|l| l.display().to_string())
                .collect::<Vec<String>>()
                .join(":")
        };

        // Last one is the main class used by mc but it needs to be forge's
        let _ = jvm_args.pop();
        // Gotta have our own branding in there
        let forge_ver = self.forge_ver.split("-").collect::<Vec<&str>>();
        let _ = mc_args.pop();
        mc_args.push(format!("forge-{}/{}", self.forge_ver, FULLNAME));
        mc_args.push("--fml.forgeGroup".to_string());
        mc_args.push("--fml.mcVersion".to_string());
        mc_args.push(self.mc_ver.clone());
        mc_args.push("net.minecraftforge".to_string());
        mc_args.push("--fml.forgeVersion".to_string());
        mc_args.push(forge_ver[1].to_string());
        mc_args.push("--launchTarget".to_string());
        mc_args.push("forge_client".to_string());

        let forge_wrapper = paths
            .iter()
            .filter(|l| {
                l.file_name()
                    .unwrap()
                    .display()
                    .to_string()
                    .contains("ForgeWrapper")
            })
            .last();

        log::debug!("{forge_wrapper:?}");
        if forge_wrapper.is_some() {
            let installer = mpaths
                .iter()
                .filter(|l| {
                    l.file_name()
                        .unwrap()
                        .display()
                        .to_string()
                        .contains("-installer")
                })
                .last();

            if let Some(i) = installer {
                let mut ver = get_versiondir();
                ver.push(format!("{}.jar", self.mc_ver));

                jvm_args.insert(
                    jvm_args.len() - 1,
                    format!("-Dforgewrapper.installer={}", i.display().to_string()),
                );
                jvm_args.insert(
                    jvm_args.len() - 1,
                    format!(
                        "-Dforgewrapper.librariesDir={}",
                        get_libdir().display().to_string()
                    ),
                );
                jvm_args.insert(
                    jvm_args.len() - 1,
                    format!("-Dforgewrapper.minecraft={}", ver.display().to_string()),
                );
            }
        }
        // else {
        //     mc_args.push("--tweakClass".to_string());
        //     mc_args.push(m.main_class.clone()); // Might help? (no it did not)
        // }

        jvm_args.push(self.main_class.clone());
        log::debug!("{jre}\n{jvm_args:#?}\n{mc_args:#?}\n{paths:#?}");

        let child = Command::new(&jre)
            .current_dir(&self.instance_dir)
            .env("CLASSPATH", &classpaths)
            .args(&jvm_args)
            .args(&mc_args)
            .spawn()
            .context(format!(
                "Failed to start minecraft with jvm {jre}\njvm_args: {jvm_args:#?}"
            ))?;

        Ok(child)
    }
}
