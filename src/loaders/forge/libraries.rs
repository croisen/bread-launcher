use std::path::{Path, PathBuf};

use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{Value, from_value};

use crate::init::{R_MINECRAFT_MVN, get_libdir};
use crate::loaders::minecraft::MinecraftLibrary;
use crate::utils::download::download;

// Hmmm it's testing for forge lib first which is 90% wrong
// So I'mma try to deserialize it myself
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ForgeLibrary {
    A(MinecraftLibrary),
    B(ForgeLibraryArtifact),
}

impl<'de> Deserialize<'de> for ForgeLibrary {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value: Value = Deserialize::deserialize(deserializer)?;
        if value.get("downloads").is_some() {
            let a: MinecraftLibrary = from_value(value).map_err(serde::de::Error::custom)?;
            Ok(ForgeLibrary::A(a))
        } else {
            let b: ForgeLibraryArtifact = from_value(value).map_err(serde::de::Error::custom)?;
            Ok(ForgeLibrary::B(b))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeLibraryArtifact {
    pub name: String,
    pub url: Option<String>,
}

impl ForgeLibraryArtifact {
    pub fn get_path(&self) -> PathBuf {
        // [0] : reverse.domain.name / group_id
        // [1] : lib_name / artifact_id
        // [2] : version
        // [3] : classifier
        let mut n = self.name.split(":");
        let domain = n.next().unwrap().replace(".", "/");
        let name = n.next().unwrap();
        let ver = n.next().unwrap();
        let mut jar = format!("{name}-{ver}");
        if let Some(cls) = n.next() {
            jar += "-";
            jar += cls;
        }

        let mut dir = get_libdir();
        dir.extend([domain.as_str(), name, ver, jar.as_str()]);
        dir.set_extension("jar");

        dir
    }

    pub fn download_library(&self, cl: Client, _instance_dir: impl AsRef<Path>) -> Result<()> {
        let base_url = if let Some(u) = self.url.as_ref() {
            u.as_str()
        } else {
            R_MINECRAFT_MVN
        };

        // [0] : reverse.domain.name / group_id
        // [1] : lib_name / artifact_id
        // [2] : version
        // [3] : classifier
        let mut n = self.name.split(":");
        let domain = n.next().unwrap().replace(".", "/");
        let name = n.next().unwrap();
        let ver = n.next().unwrap();
        let mut jar = format!("{name}-{ver}");
        if let Some(cls) = n.next() {
            jar += "-";
            jar += cls;
        }

        let url = format!("{base_url}/{domain}/{name}/{ver}/{jar}.jar",);
        let mut dir = get_libdir();
        dir.extend([domain.as_str(), name, ver]);
        download(&cl, dir, format!("{jar}.jar"), url, 1)?;

        Ok(())
    }
}
