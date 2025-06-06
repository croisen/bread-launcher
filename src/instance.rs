use std::collections::BTreeMap;
use std::fs::{read, remove_file, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};

use crate::minecraft::MinecraftVersionManifest;
use crate::minecraft::{MVOrganized, Minecraft};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instances {
    col: BTreeMap<String, Arc<Instance>>,
    last_check: u64,

    #[serde(skip_serializing, skip_deserializing)]
    cl: Client,
    #[serde(skip_serializing, skip_deserializing)]
    versions: MVOrganized,
}

impl Instances {
    pub async fn new(cl: Client, appdir: impl AsRef<Path>) -> Result<Self> {
        let vm = appdir.as_ref().join("version_manifest_v2.json");
        let instances = appdir.as_ref().join("instances.json");
        let t = SystemTime::now().duration_since(UNIX_EPOCH)?;
        // Re-download version manifest if 10 days has passed
        let r = Duration::new(10 * 24 * 60 * 60, 0);

        let s = if instances.is_file() {
            let c = read(&instances)?;
            let mut de = Deserializer::from_slice(c.as_ref());
            let mut s = Self::deserialize(&mut de)?;
            log::info!(
                "Instance collection found, checking if it hit the set expiration (default: 10 days)"
            );

            let d = Duration::from_secs(s.last_check);
            if let Some(ts) = t.checked_sub(d) {
                if ts.as_secs() > r.as_secs() {
                    remove_file(&vm)?;
                    log::info!("Triggering re-download of version manifest...");
                    s.last_check = t.as_secs();
                }
            }

            s.versions = MinecraftVersionManifest::new(&cl, appdir.as_ref())
                .await?
                .into();
            s.cl = cl;

            s
        } else {
            if vm.exists() {
                remove_file(&vm)?;
                log::info!(
                    "No instance collection found, triggering re-download of version manifest..."
                );
            }

            Self {
                col: BTreeMap::new(),
                last_check: 0,

                cl: cl.clone(),
                versions: MinecraftVersionManifest::new(&cl, appdir.as_ref())
                    .await?
                    .into(),
            }
        };

        Ok(s)
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

    pub async fn save(&self, appdir: impl AsRef<Path>) -> Result<()> {
        let mut se = Serializer::pretty(
            OpenOptions::new()
                .write(true)
                .create(true)
                .open(appdir.as_ref().join("instances.json"))?,
        );

        self.serialize(&mut se)?;
        Ok(())
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

    #[serde(skip_serializing, skip_deserializing)]
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

    pub async fn run(&self, ram: String, username: String) -> Result<()> {
        let m = Minecraft::new(self.path.as_ref())?;
        m.run(self.cl.clone(), ram, username).await?;
        Ok(())
    }
}
