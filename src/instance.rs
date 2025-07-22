use anyhow::{Result, anyhow, bail};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::remove_file as tk_remove_file;
use tokio::fs::rename as tk_rename;

use crate::account::Account;
use crate::minecraft::MinecraftVersionManifest;
use crate::minecraft::{MVOrganized, Minecraft};

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

        let cp = v.download(&self.cl, appdir.as_ref()).await?;
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
