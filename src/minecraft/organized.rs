use std::collections::BTreeMap;
use std::sync::Arc;

use crate::minecraft::version_manifest::MinecraftVersion;
use crate::minecraft::MinecraftVersionManifest;

#[derive(Default, Debug, Clone)]
pub struct MVOrganized {
    pub release: BTreeMap<String, Arc<MinecraftVersion>>,
    pub snapshot: BTreeMap<String, Arc<MinecraftVersion>>,
    pub beta: BTreeMap<String, Arc<MinecraftVersion>>,
    pub alpha: BTreeMap<String, Arc<MinecraftVersion>>,
}

impl MVOrganized {
    pub fn new(mvm: MinecraftVersionManifest) -> Self {
        let mut release = BTreeMap::new();
        let mut snapshot = BTreeMap::new();
        let mut beta = BTreeMap::new();
        let mut alpha = BTreeMap::new();

        for ver in mvm.versions {
            match ver.version_type.as_ref() {
                "release" => {
                    release.insert(ver.id.to_string(), Arc::new(ver));
                }
                "snapshot" => {
                    snapshot.insert(ver.id.to_string(), Arc::new(ver));
                }
                "old_beta" => {
                    beta.insert(ver.id.to_string(), Arc::new(ver));
                }
                "old_alpha" => {
                    alpha.insert(ver.id.to_string(), Arc::new(ver));
                }
                _ => {}
            }
        }

        Self {
            release,
            snapshot,
            beta,
            alpha,
        }
    }
}

impl From<MinecraftVersionManifest> for MVOrganized {
    fn from(value: MinecraftVersionManifest) -> Self {
        Self::new(value)
    }
}
