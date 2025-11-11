use std::path::PathBuf;

use serde::{Deserialize, Serialize};

mod version_manifest;

pub use version_manifest::ForgeVersionManifest;

use crate::loaders::minecraft::MinecraftLibrary;

#[derive(Debug, Serialize, Deserialize)]
pub struct Forge {
    minecraft: String,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    libraries: Vec<MinecraftLibrary>,

    #[serde(skip)]
    instance_dir: PathBuf,
}
