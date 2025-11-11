use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::read;

use anyhow::{Context, Result, anyhow};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::from_slice;

use crate::init::{L_FORGE_REC, L_FORGE_VER, R_FORGE_REC, R_FORGE_VER, get_appdir};
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

        let f = read(&forge_vers).context(anyhow!(
            "Failed to read forge versions from {forge_vers:#?}"
        ))?;

        let g = read(&forge_recs).context(anyhow!(
            "Failed to read forge recommendations from {forge_recs:#?}"
        ))?;

        Ok(Self {
            versions: from_slice(f.as_slice())?,
            recommends: from_slice(g.as_slice())?,
        })
    }
}
