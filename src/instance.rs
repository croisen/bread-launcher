use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::fs::{create_dir_all, remove_file, rename};
use std::mem::swap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

use anyhow::{Result, bail};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::account::Account;
use crate::init::get_appdir;
use crate::minecraft::MinecraftVersionManifest;
use crate::minecraft::{MVOrganized, Minecraft};
use crate::utils::message::Message;

// I'm gonna think of something else or I'll just let it be
pub static UNGROUPED_NAME: &str = "Venator A Mi Sumo Vela Mala";
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Instances {
    // Group name, Instance Name, Instance
    col: BTreeMap<String, BTreeMap<String, Arc<Mutex<Instance>>>>,

    #[serde(skip)]
    pub cl: Client,
    #[serde(skip)]
    versions: MVOrganized,
}

impl Instances {
    pub fn new(cl: Client) -> Result<Self> {
        let mvm = MinecraftVersionManifest::new(&cl)?;

        Ok(Self {
            col: BTreeMap::new(),
            cl: cl.clone(),
            versions: MVOrganized::new(&mvm),
        })
    }

    pub fn parse_versions(&mut self) -> Result<()> {
        self.versions = self.versions.renew(&self.cl)?;
        Ok(())
    }

    pub fn renew_version(&mut self) -> Result<()> {
        let appdir = get_appdir();
        let vm = appdir.join("version_manifest_v2.json");
        let rvm = appdir.join("version_manifest_v2.json.bak");
        let exists = vm.is_file();
        if exists {
            rename(&vm, &rvm)?;
        }

        match self.versions.renew(&self.cl) {
            Ok(mvo) => {
                if rvm.exists() {
                    remove_file(&rvm)?;
                }

                self.versions = mvo;
                Ok(())
            }
            Err(e) => {
                if exists {
                    log::error!("Could not renew minecraft version manifest");
                    rename(&rvm, &vm)?;
                } else {
                    log::error!("Could not download minecraft version manifest");
                }

                Err(e)
            }
        }
    }

    pub fn add_instance(&mut self, group_name: impl AsRef<str>, instance: Instance) {
        let group_name = if group_name.as_ref().is_empty() {
            UNGROUPED_NAME.to_string()
        } else {
            group_name.as_ref().to_string()
        };

        if let Some(instances) = self.col.get_mut::<str>(group_name.as_ref()) {
            instances.insert(
                instance.name.as_ref().to_string(),
                Mutex::new(instance).into(),
            );
        } else {
            let mut instances = BTreeMap::new();
            instances.insert(
                instance.name.as_ref().to_string(),
                Mutex::new(instance).into(),
            );
            self.col.insert(group_name, instances);
        };
    }

    pub fn get_instances(&self) -> &BTreeMap<String, BTreeMap<String, Arc<Mutex<Instance>>>> {
        &self.col
    }

    pub fn get_versions(&self) -> &MVOrganized {
        &self.versions
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstanceLoader {
    #[default]
    Vanilla,
    Forge,
    Fabric,
    Forgelite,
    Quilt,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Instance {
    pub name: Arc<str>,
    pub mc_ver: Arc<str>,
    pub full_ver: Arc<str>, // Getting ready for different mod loader versions
    path: Arc<PathBuf>,
    pub loader: InstanceLoader,

    #[serde(skip)]
    run: Option<JoinHandle<Result<Child>>>,
}

impl Instance {
    pub fn new(
        name: impl AsRef<str>,
        mc_ver: impl AsRef<str>,
        full_ver: impl AsRef<str>,
        path: Arc<PathBuf>,
        loader: InstanceLoader,
    ) -> Self {
        Self {
            name: name.as_ref().into(),
            mc_ver: mc_ver.as_ref().into(),
            full_ver: full_ver.as_ref().into(),
            path,
            loader,
            run: None,
        }
    }

    pub fn run_offline(
        &mut self,
        cl: Client,
        _step: Arc<AtomicUsize>,
        _total_steps: Arc<AtomicUsize>,
        tx: Sender<Message>,
        ram: usize,
        account: Arc<Mutex<Account>>,
    ) -> Result<()> {
        create_dir_all(self.path.as_ref())?;
        let account = account.lock().unwrap().clone().into();
        let loader = self.loader;
        let path = self.path.clone();
        let mc_ver = self.mc_ver.clone();
        let _ = tx.send(Message::msg("Now launching instance"));

        self.run = Some(spawn::<_, Result<Child>>(move || {
            let child = match loader {
                InstanceLoader::Vanilla => {
                    let m = Minecraft::new(path.as_ref(), mc_ver)?;
                    m.run(cl.clone(), ram, account)
                }
                _ => bail!("Unimplemented"),
            };

            child
        }));

        Ok(())
    }

    pub fn run(
        &mut self,
        cl: Client,
        step: Arc<AtomicUsize>,
        total_steps: Arc<AtomicUsize>,
        tx: Sender<Message>,
        ram: usize,
        account: Arc<Mutex<Account>>,
    ) -> Result<()> {
        create_dir_all(self.path.as_ref())?;
        let account = account.lock().unwrap().clone().into();
        let loader = self.loader;
        let path = self.path.clone();
        let mc_ver = self.mc_ver.clone();

        self.run = Some(spawn::<_, Result<Child>>(move || {
            let child = match loader {
                InstanceLoader::Vanilla => {
                    let m = Minecraft::new(path.as_ref(), mc_ver)?;
                    m.download_jre(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                    m.download_client(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                    m.download_assets(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                    let _ = tx.send(Message::msg("Now launching instance"));
                    m.run(cl.clone(), ram, account)
                }
                _ => bail!("Unimplemented"),
            };

            child
        }));

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        let r = if let Some(run) = &self.run {
            !run.is_finished()
        } else {
            false
        };

        r
    }

    pub fn stop(&mut self) {
        let mut run = None;
        swap(&mut self.run, &mut run);
        if let Some(run) = run {
            let chld = run.join().unwrap();
            if let Ok(mut chld) = chld {
                let _ = chld.kill();
            }
        }
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        let a = self.name == other.name;
        let b = self.mc_ver == other.mc_ver;
        let c = self.full_ver == other.full_ver;
        let d = self.loader == other.loader;
        let e = self.path == other.path;

        a && b && c && d && e
    }
}
