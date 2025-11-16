use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::{read_to_string, remove_file, rename};
use std::mem::swap;

use anyhow::{Context, Result, anyhow};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

use crate::init::{L_FORGE_REC, L_FORGE_VER, R_FORGE_REC, R_FORGE_VER, get_appdir, get_forge_path};
use crate::utils::download::download;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ForgeRecommends {
    pub homepage: String,
    pub promos: HashMap<String, String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ForgeVersionManifest {
    pub versions: HashMap<String, Vec<String>>,
    pub recommends: ForgeRecommends,
}

impl ForgeVersionManifest {
    pub fn new(cl: Client) -> Result<Self> {
        let mut forge_vers = get_appdir();
        let mut forge_recs = get_appdir();
        forge_vers.extend(["loaders", L_FORGE_VER]);
        forge_recs.extend(["loaders", L_FORGE_REC]);
        if !forge_vers.is_file() {
            let _ = forge_vers.pop();
            download(&cl, &forge_vers, L_FORGE_VER, R_FORGE_VER, 1)?;
            forge_vers.push(L_FORGE_VER);
        }

        if !forge_recs.is_file() {
            let _ = forge_recs.pop();
            download(&cl, &forge_recs, L_FORGE_REC, R_FORGE_REC, 1)?;
            forge_recs.push(L_FORGE_REC);
        }

        let f = read_to_string(&forge_vers).context(anyhow!(
            "Failed to read forge versions from {forge_vers:#?}"
        ))?;

        let g = read_to_string(&forge_recs).context(anyhow!(
            "Failed to read forge recommendations from {forge_recs:#?}"
        ))?;

        let mut versions: HashMap<String, Vec<String>> = from_str(&f)?;
        for v in versions.values_mut() {
            v.reverse();
        }

        if let Some(odd_ones) = versions.get_mut("1.7.10") {
            for odd in odd_ones.iter_mut() {
                // The others have extra bits at the end such as -1.7.10 ot -new
                // we're removing that
                odd.truncate("1.7.10-10.13.4.1614".len());
            }
        }

        Ok(Self {
            versions,
            recommends: from_str(&g)?,
        })
    }

    pub fn renew(&mut self, cl: Client) -> Result<()> {
        let mut fvm = ForgeVersionManifest::new(cl)?;
        swap(self, &mut fvm);

        Ok(())
    }

    pub fn redownload(&mut self, cl: Client) -> Result<()> {
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

pub fn download_forge_json(
    cl: Client,
    mc_ver: impl AsRef<str>,
    forge_ver: impl AsRef<str>,
) -> Result<()> {
    let name = format!("forge-{}.json", forge_ver.as_ref());
    let ver = forge_ver
        .as_ref()
        .trim_prefix(&format!("{}-", mc_ver.as_ref()));

    let mut p = get_forge_path(&forge_ver);
    let _ = p.pop();
    let url = format!(
        "https://raw.githubusercontent.com/MultiMC/meta-multimc/refs/heads/master/net.minecraftforge/{ver}.json"
    );

    download(&cl, p, name, url, 1)?;
    Ok(())
}
