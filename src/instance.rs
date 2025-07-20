use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::remove_file as tk_remove_file;
use tokio::fs::rename as tk_rename;

use crate::account::Account;
use crate::minecraft::MinecraftVersionManifest;
use crate::minecraft::{MVOrganized, Minecraft};

// I'm gonna think of something else or I'll just let it be
pub static UNGROUPED_NAME: &str = "Venator A Mi Sumo Vela Mala";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instances {
    // Group name, Instance Name, Instance
    col: BTreeMap<String, BTreeMap<String, Arc<Instance>>>,

    #[serde(skip)]
    cl: Client,
    #[serde(skip)]
    versions: MVOrganized,
}

impl Instances {
    pub async fn new(cl: Client, appdir: impl AsRef<Path>) -> Result<Self> {
        let mvm = MinecraftVersionManifest::new(&cl, appdir.as_ref()).await?;
        log::info!("Version manifest length: {}", mvm.versions.len());

        Ok(Self {
            col: BTreeMap::new(),
            cl: cl.clone(),
            versions: MVOrganized::new(&mvm),
        })
    }

    pub async fn parse_versions(&mut self, appdir: impl AsRef<Path>) -> Result<()> {
        self.versions = self.versions.renew(&self.cl, appdir.as_ref()).await?;
        Ok(())
    }

    pub async fn renew_version(&mut self, appdir: impl AsRef<Path>) -> Result<()> {
        let vm = appdir.as_ref().join("version_manifest_v2.json");
        let rvm = appdir.as_ref().join("version_manifest_v2.json.bak");
        let exists = vm.is_file();
        if exists {
            tk_rename(&vm, &rvm).await?;
        }

        match self.versions.renew(&self.cl, appdir.as_ref()).await {
            Ok(mvo) => {
                if rvm.exists() {
                    tk_remove_file(&rvm).await?;
                }

                self.versions = mvo;
                Ok(())
            }
            Err(e) => {
                if exists {
                    log::error!("Could not renew minecraft version manifest");
                    tk_rename(&rvm, &vm).await?;
                } else {
                    log::error!("Could not download minecraft version manifest");
                }

                Err(e)
            }
        }
    }

    pub async fn new_instance(
        &mut self,
        appdir: impl AsRef<Path>,
        rel_type: &str,
        version: &Arc<str>,
        group_name: &str,
        name: &str,
        loader: InstanceLoader,
    ) -> Result<Arc<Instance>> {
        log::info!("Release count: {}", self.versions.release.len());
        log::info!("Snapshot count: {}", self.versions.snapshot.len());
        log::info!("Beta count: {}", self.versions.beta.len());
        log::info!("Alpha count: {}", self.versions.alpha.len());

        let v = match rel_type {
            "release" => self
                .versions
                .release
                .iter()
                .filter(|x| x.id == version.clone())
                .take(1)
                .next()
                .ok_or(anyhow!("Release version {version} not found..."))?,
            "snapshot" => self
                .versions
                .snapshot
                .iter()
                .filter(|x| x.id == version.clone())
                .take(1)
                .next()
                .ok_or(anyhow!("Snapshot version {version} not found..."))?,
            "old_beta" => self
                .versions
                .beta
                .iter()
                .filter(|x| x.id == version.clone())
                .take(1)
                .next()
                .ok_or(anyhow!("Beta version {version} not found..."))?,
            "old_alpha" => self
                .versions
                .alpha
                .iter()
                .filter(|x| x.id == version.clone())
                .take(1)
                .next()
                .ok_or(anyhow!("Alpha version {version} not found..."))?,
            _ => {
                return Err(anyhow!("What kinda release type is this: {rel_type}?"));
            }
        };

        let cp = v.download(&self.cl, appdir.as_ref()).await?;
        let m = Minecraft::new(cp, version.clone())?;
        let i = m.new_instance()?;
        let instance = Arc::new(Instance::new(
            self.cl.clone(),
            name,
            version,
            i.get_cache_dir(),
            loader,
        ));

        let group_name = if group_name.is_empty() {
            UNGROUPED_NAME.to_string()
        } else {
            group_name.to_string()
        };

        if let Some(instances) = self.col.get_mut(&group_name) {
            instances.insert(name.to_string(), instance.clone());
        } else {
            let mut instances = BTreeMap::new();
            instances.insert(name.to_string(), instance.clone());
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
    name: Arc<str>,
    version: Arc<str>,
    path: Arc<PathBuf>,
    loader: InstanceLoader,

    #[serde(skip)]
    cl: Client,
}

impl Instance {
    fn new(
        cl: Client,
        name: &str,
        version: &str,
        path: Arc<PathBuf>,
        loader: InstanceLoader,
    ) -> Self {
        Self {
            name: Arc::from(name),
            version: Arc::from(version),
            path,
            loader,
            cl,
        }
    }

    pub async fn run(&self, ram: String, account: Arc<Account>) -> Result<()> {
        let m = Minecraft::new(self.path.as_ref(), self.version.clone())?;
        m.run(self.cl.clone(), ram, account).await?;

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
