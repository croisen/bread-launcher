use std::fs::{remove_file, rename};
use std::mem::swap;
use std::sync::Arc;

use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::init::{L_FORGE_VER, get_appdir};
use crate::loaders::minecraft::{MinecraftVersion, MinecraftVersionManifest};

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
pub struct MVOrganized {
    pub release: Vec<Arc<MinecraftVersion>>,
    pub snapshot: Vec<Arc<MinecraftVersion>>,
    pub beta: Vec<Arc<MinecraftVersion>>,
    pub alpha: Vec<Arc<MinecraftVersion>>,
}

impl MVOrganized {
    pub fn new(mvm: &MinecraftVersionManifest) -> Self {
        let mut release = Vec::new();
        let mut snapshot = Vec::new();
        let mut beta = Vec::new();
        let mut alpha = Vec::new();

        for ver in &mvm.versions {
            match ver.version_type.as_ref() {
                "release" => {
                    release.push(ver.clone());
                }
                "snapshot" => {
                    snapshot.push(ver.clone());
                }
                "old_beta" => {
                    beta.push(ver.clone());
                }
                "old_alpha" => {
                    alpha.push(ver.clone());
                }
                _ => {
                    log::error!("Unknown version: {} {}", ver.id, ver.version_type);
                }
            }
        }

        Self {
            release,
            snapshot,
            beta,
            alpha,
        }
    }

    pub fn renew(&mut self, cl: Client) -> Result<()> {
        let mut mvm = MinecraftVersionManifest::new(cl)?.into();
        swap(self, &mut mvm);

        Ok(())
    }

    pub fn renew_version(&mut self, cl: Client) -> Result<()> {
        let appdir = get_appdir().join("loaders");
        let vm = appdir.join(L_FORGE_VER);
        let rvm = appdir.join(format!("{L_FORGE_VER}.bak"));
        let exists = vm.is_file();
        if exists {
            rename(&vm, &rvm)?;
        }

        match self.renew(cl) {
            Ok(_) => {
                if rvm.exists() {
                    remove_file(&rvm)?;
                }

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
}

impl From<MinecraftVersionManifest> for MVOrganized {
    fn from(value: MinecraftVersionManifest) -> Self {
        Self::new(&value)
    }
}
