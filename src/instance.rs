use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::atomic::AtomicUsize;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, spawn};

use anyhow::{Result, bail};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::account::Account;
use crate::init::UNGROUPED_NAME;
use crate::minecraft::{Minecraft, MinecraftVersion};
use crate::utils::message::Message;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Instances {
    // Group name, Instance Name, Instance
    col: BTreeMap<String, BTreeMap<String, Arc<Mutex<Instance>>>>,
}

impl Instances {
    pub fn new() -> Self {
        Self {
            col: BTreeMap::new(),
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

    pub fn new_vanilla_instance(
        cl: Client,
        name: impl AsRef<str>,
        mc_ver: impl AsRef<str>,
        full_ver: impl AsRef<str>,
        version: Arc<MinecraftVersion>,
    ) -> Result<Instance> {
        version.download(&cl)?;
        let m = Minecraft::new(Path::new("a"), &mc_ver)?;
        let c = m.new_instance()?;
        let instance = Instance::new(
            name,
            mc_ver,
            full_ver,
            c.get_cache_dir(),
            InstanceLoader::Vanilla,
        );

        Ok(instance)
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstanceLoader {
    #[default]
    Vanilla = 0,
    Forge = 1,
    Fabric = 2,
    LiteLoader = 3,
    Quilt = 4,
}

impl From<InstanceLoader> for usize {
    fn from(value: InstanceLoader) -> Self {
        value as Self
    }
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
    #[serde(skip)]
    child: Option<Child>,
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
            child: None,
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

        self.run = Some(spawn::<_, Result<Child>>(move || match loader {
            InstanceLoader::Vanilla => {
                let m = Minecraft::new(path.as_ref(), mc_ver)?;
                m.run(cl.clone(), ram, account)
            }
            InstanceLoader::Forge => bail!("Unimplemented"),
            InstanceLoader::LiteLoader => bail!("Unimplemented"),
            InstanceLoader::Fabric => bail!("Unimplemented"),
            InstanceLoader::Quilt => bail!("Unimplemented"),
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

        self.run = Some(spawn::<_, Result<Child>>(move || match loader {
            InstanceLoader::Vanilla => {
                let m = Minecraft::new(path.as_ref(), mc_ver)?;
                m.download_jre(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                m.download_client(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                m.download_assets(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                let _ = tx.send(Message::msg("Now launching instance"));
                m.run(cl.clone(), ram, account)
            }
            InstanceLoader::Forge => bail!("Unimplemented"),
            InstanceLoader::LiteLoader => bail!("Unimplemented"),
            InstanceLoader::Fabric => bail!("Unimplemented"),
            InstanceLoader::Quilt => bail!("Unimplemented"),
        }));

        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        match &mut self.child {
            Some(child) => match child.try_wait() {
                Ok(Some(_)) => {
                    self.child = None;
                    false
                }
                Ok(None) => true,
                Err(e) => {
                    log::error!("Error in checking if child process is running: {e}");
                    false
                }
            },
            None => match &self.run {
                Some(run) => {
                    if run.is_finished() {
                        let res = self.run.take().unwrap();
                        let child = res.join().unwrap();
                        if let Ok(chld) = child {
                            self.child = Some(chld);
                            return self.is_running();
                        }
                    }

                    true
                }
                None => false,
            },
        }
    }

    /// This would hang if it's still downloading tho
    /// TODO stop this from hanging (by using tokio??? or more channels)
    pub fn stop(&mut self) {
        if self.run.is_some() {
            let thread = self.run.take().unwrap();
            self.child = thread.join().unwrap().ok();
        }

        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = self.child.take();
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
