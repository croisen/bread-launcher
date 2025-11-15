use std::path::{Path, PathBuf};

use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::init::get_libdir;
use crate::loaders::minecraft::MinecraftLibrary;
use crate::utils::download::download;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForgeLibrary {
    M(MinecraftLibrary),
    F(ForgeLibraryArtifact),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeLibraryArtifact {
    pub name: String,
    pub url: Option<String>,
}

impl ForgeLibraryArtifact {
    pub fn get_path(&self) -> Option<PathBuf> {
        if self.url.is_none() {
            return None;
        }

        // [0] : reverse.domain.name / group_id
        // [1] : lib_name / artifact_id
        // [2] : version
        // [3] : whatever?
        let mut n = self.name.split(":");
        let full = format!("{},jar", self.name.replace(":", "-"));
        let domain = n.next().unwrap().replace(".", "/");
        let name = n.next().unwrap();
        let ver = n.next().unwrap();

        let mut dir = get_libdir();
        dir.extend([domain.as_str(), name, ver, full.as_str()]);

        Some(dir)
    }

    pub fn download_library(&self, cl: Client, _instance_dir: impl AsRef<Path>) -> Result<()> {
        if self.url.is_none() {
            return Ok(());
        }

        // [0] : reverse.domain.name / group_id
        // [1] : lib_name / artifact_id
        // [2] : version
        // [3] : whatever?
        let mut n = self.name.split(":");
        let full = format!("{},jar", self.name.replace(":", "-"));
        let domain = n.next().unwrap().replace(".", "/");
        let name = n.next().unwrap();
        let ver = n.next().unwrap();
        let url = format!(
            "{}/{}/{}/{}/{}",
            self.url.as_ref().unwrap(),
            domain,
            name,
            ver,
            full
        );

        let mut dir = get_libdir();
        dir.extend([domain.as_str(), name, ver]);
        download(&cl, dir, full, url, 1)?;

        Ok(())
    }
}
