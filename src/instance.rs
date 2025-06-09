use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::remove_file as tk_remove_file;
use tokio::fs::rename as tk_rename;

use crate::minecraft::MinecraftVersionManifest;
use crate::minecraft::{MVOrganized, Minecraft};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instances {
    col: BTreeMap<String, Arc<Instance>>,

    #[serde(skip)]
    cl: Client,
    #[serde(skip)]
    versions: MVOrganized,
}

impl Instances {
    pub async fn new(cl: Client, appdir: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            col: BTreeMap::new(),
            cl: cl.clone(),
            versions: MinecraftVersionManifest::new(&cl, appdir.as_ref())
                .await?
                .into(),
        })
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
        app_dir: impl AsRef<Path>,
        rel_type: &str,
        version: &str,
        name: &str,
        loader: InstanceLoader,
    ) -> Result<Arc<Instance>> {
        let v = match rel_type {
            "release" => self
                .versions
                .release
                .get(version)
                .ok_or(anyhow!("Release version {version} not found...")),
            "snapshot" => self
                .versions
                .snapshot
                .get(version)
                .ok_or(anyhow!("Snapshot version {version} not found...")),
            "beta" => self
                .versions
                .beta
                .get(version)
                .ok_or(anyhow!("Beta version {version} not found...")),
            "alpha" => self
                .versions
                .alpha
                .get(version)
                .ok_or(anyhow!("Alpha version {version} not found...")),
            _ => {
                return Err(anyhow!("What kinda release type is this: {rel_type}?"));
            }
        };

        match v {
            Ok(vv) => {
                let cp = vv.download(&self.cl, app_dir.as_ref()).await?;
                let m = Minecraft::new(cp)?;
                let _ = m.download(&self.cl).await?;
                let i = m.new_insatance()?;
                let instance = Arc::new(Instance::new(
                    self.cl.clone(),
                    name,
                    version,
                    i.get_cache_dir(),
                    loader,
                ));

                self.col.insert(name.to_string(), instance.clone());

                Ok(instance)
            }
            Err(e) => Err(e),
        }
    }

    pub fn get_instance(&self, name: &str) -> Result<&Arc<Instance>> {
        self.col
            .get(name)
            .ok_or(anyhow!("Instance named {name} not found"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceLoader {
    Vanilla,
    Forge,
    Fabric,
    Forgelite,
    Quilt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub async fn run(
        &self,
        ram: String,
        username: String,
        access_token: String,
        user_properties: String,
    ) -> Result<()> {
        let m = Minecraft::new(self.path.as_ref())?;
        m.run(
            self.cl.clone(),
            ram,
            username,
            access_token,
            user_properties,
        )
        .await?;

        Ok(())
    }
}
