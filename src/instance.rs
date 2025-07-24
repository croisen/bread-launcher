use std::fs::{create_dir_all, remove_file, rename};
use std::sync::atomic::AtomicUsize;
use std::sync::mpmc::Sender;

use anyhow::{Result, anyhow, bail};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::account::Account;
use crate::minecraft::MinecraftVersionManifest;
use crate::minecraft::{MVOrganized, Minecraft};
use crate::utils::message::Message;

// I'm gonna think of something else or I'll just let it be
pub static UNGROUPED_NAME: &str = "Venator A Mi Sumo Vela Mala";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Instances {
    // Group name, Instance Name, Instance
    col: BTreeMap<String, BTreeMap<String, Arc<Instance>>>,

    #[serde(skip)]
    pub cl: Client,
    #[serde(skip)]
    versions: MVOrganized,
}

impl Instances {
    pub fn new(cl: Client, appdir: impl AsRef<Path>) -> Result<Self> {
        let mvm = MinecraftVersionManifest::new(&cl, appdir.as_ref())?;
        log::info!("Version manifest length: {}", mvm.versions.len());

        Ok(Self {
            col: BTreeMap::new(),
            cl: cl.clone(),
            versions: MVOrganized::new(&mvm),
        })
    }

    pub fn parse_versions(&mut self, appdir: impl AsRef<Path>) -> Result<()> {
        self.versions = self.versions.renew(&self.cl, appdir.as_ref())?;
        Ok(())
    }

    pub fn renew_version(&mut self, appdir: impl AsRef<Path>) -> Result<()> {
        let vm = appdir.as_ref().join("version_manifest_v2.json");
        let rvm = appdir.as_ref().join("version_manifest_v2.json.bak");
        let exists = vm.is_file();
        if exists {
            rename(&vm, &rvm)?;
        }

        match self.versions.renew(&self.cl, appdir.as_ref()) {
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

    pub fn new_instance(
        &mut self,
        appdir: impl AsRef<Path>,
        rel_type: impl AsRef<str>,
        version: impl AsRef<str>,
        group_name: impl AsRef<str>,
        name: impl AsRef<str>,
        loader: InstanceLoader,
    ) -> Result<Arc<Instance>> {
        log::info!("Release count: {}", self.versions.release.len());
        log::info!("Snapshot count: {}", self.versions.snapshot.len());
        log::info!("Beta count: {}", self.versions.beta.len());
        log::info!("Alpha count: {}", self.versions.alpha.len());

        let v = match rel_type.as_ref() {
            "release" => self
                .versions
                .release
                .iter()
                .filter(|x| x.id == version.as_ref().into())
                .take(1)
                .next()
                .ok_or(anyhow!("Release version {} not found...", version.as_ref()))?,
            "snapshot" => self
                .versions
                .snapshot
                .iter()
                .filter(|x| x.id == version.as_ref().into())
                .take(1)
                .next()
                .ok_or(anyhow!(
                    "Snapshot version {} not found...",
                    version.as_ref()
                ))?,
            "old_beta" => self
                .versions
                .beta
                .iter()
                .filter(|x| x.id == version.as_ref().into())
                .take(1)
                .next()
                .ok_or(anyhow!("Beta version {} not found...", version.as_ref()))?,
            "old_alpha" => self
                .versions
                .alpha
                .iter()
                .filter(|x| x.id == version.as_ref().into())
                .take(1)
                .next()
                .ok_or(anyhow!("Alpha version {} not found...", version.as_ref()))?,
            _ => {
                bail!("What kinda release type is this: {}?", rel_type.as_ref());
            }
        };

        let cp = v.download(&self.cl, appdir.as_ref())?;
        let m = Minecraft::new(cp, version.as_ref())?;
        let i = m.new_instance()?;
        let instance = Arc::new(Instance::new(
            self.cl.clone(),
            name.as_ref(),
            version,
            i.get_cache_dir(),
            loader,
        ));

        let group_name = if group_name.as_ref().is_empty() {
            UNGROUPED_NAME.to_string()
        } else {
            group_name.as_ref().to_string()
        };

        if let Some(instances) = self.col.get_mut::<str>(group_name.as_ref()) {
            instances.insert(name.as_ref().to_string(), instance.clone());
        } else {
            let mut instances = BTreeMap::new();
            instances.insert(name.as_ref().to_string(), instance.clone());
            self.col.insert(group_name, instances);
        }

        Ok(instance)
    }

    pub fn get_instance(&self, group: &str, name: &str) -> Result<Arc<Instance>> {
        let instance = self
            .col
            .get(group)
            .ok_or(anyhow!("Group for instances {group:?} not found"))?
            .get(name)
            .ok_or(anyhow!("Instance named {name} not found"))?
            .clone();

        Ok(instance)
    }

    pub fn get_instances(&self) -> &BTreeMap<String, BTreeMap<String, Arc<Instance>>> {
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

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub name: Arc<str>,
    pub version: Arc<str>,
    path: Arc<PathBuf>,
    pub loader: InstanceLoader,

    #[serde(skip)]
    cl: Client,
}

impl Instance {
    fn new(
        cl: Client,

        name: impl AsRef<str>,
        version: impl AsRef<str>,
        path: Arc<PathBuf>,
        loader: InstanceLoader,
    ) -> Self {
        Self {
            name: Arc::from(name.as_ref()),
            version: Arc::from(version.as_ref()),
            path,
            loader,
            cl,
        }
    }

    pub fn run_offline(&self, ram: usize, account: Arc<Account>) -> Result<()> {
        create_dir_all(self.path.as_ref())?;
        match self.loader {
            InstanceLoader::Vanilla => {
                let m = Minecraft::new(self.path.as_ref(), self.version.clone())?;
                m.run(self.cl.clone(), ram, account)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn run(
        &self,
        cl: Client,
        step: Arc<AtomicUsize>,
        total_steps: Arc<AtomicUsize>,
        tx: Sender<Message>,
        ram: usize,
        account: Arc<Account>,
    ) -> Result<()> {
        create_dir_all(self.path.as_ref())?;
        match self.loader {
            InstanceLoader::Vanilla => {
                let m = Minecraft::new(self.path.as_ref(), self.version.clone())?;
                m.download_jre(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                m.download_client(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                m.download_assets(cl.clone(), step.clone(), total_steps.clone(), tx.clone())?;
                let _ = tx.send(Message::Message("Now launching instance".to_string()));
                m.run(self.cl.clone(), ram, account)?;
            }
            _ => {}
        }

        Ok(())
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        let a = self.name == other.name;
        let b = self.version == other.version;
        let c = self.loader == other.loader;
        let d = self.path == other.path;

        a && b && c && d
    }
}
