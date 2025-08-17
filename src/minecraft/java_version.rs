use std::env::consts;
use std::ffi::OsStr;
use std::fs::{File, create_dir_all, remove_dir, remove_file};
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tar::Archive;
use zip::read::{ZipArchive, root_dir_common_filter};

use crate::init::{get_javadir, get_tempdir};
use crate::utils::download::download;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftJavaVersion {
    component: Arc<str>,
    #[serde(rename = "majorVersion")]
    major_version: usize,
}

impl MinecraftJavaVersion {
    pub fn check_exists(&self) -> bool {
        let mut j = get_javadir();
        j.push(format!("{:0>2}", self.major_version));
        j.push("bin");

        j.is_dir()
    }

    pub fn get_version(&self) -> usize {
        self.major_version
    }

    pub fn download(&self, cl: &Client) -> Result<()> {
        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else {
            consts::ARCH
        };

        // We're using Temurin
        let jre: String = format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse?project=jdk",
            self.major_version,
            consts::OS,
            arch,
        );

        let mut j = get_javadir();
        j.push(format!("{:0>2}", self.major_version));
        j.push("bin");
        if !j.is_dir() {
            let _ = j.pop();
            let mut t = get_tempdir();
            // Not exactly a zip in all platforms but I'm feeling lazy
            download(cl, &t, "temurin.zip", &jre, 1)?;
            t.push("temurin.zip");
            let f = File::open(&t).context("Could not open downloaded jre archive?")?;
            if cfg!(target_family = "windows") {
                ZipArchive::new(f)?.extract_unwrapped_root_dir(&j, root_dir_common_filter)?;
            } else {
                self.extract_unwrapped_root_dir(f, &j)?;
            }

            remove_file(&t)?;
            let _ = t.pop();
            remove_dir(&t)?;
        }

        Ok(())
    }

    fn extract_unwrapped_root_dir(&self, f: File, dir: impl AsRef<Path>) -> Result<()> {
        let gz = GzDecoder::new(f);
        let mut tar = Archive::new(gz);
        for entry in tar.entries().context("Could not get jre archive entries")? {
            match entry {
                Ok(mut e) => {
                    let etype = e.header().entry_type();
                    match etype {
                        tar::EntryType::Regular => {}
                        _ => continue,
                    }

                    let fpath = e.path()?;
                    let f = fpath.as_ref().iter().collect::<Vec<&OsStr>>();
                    let f_name = f
                        .last()
                        .ok_or(anyhow!("Could not get file names from the java archive"))?;

                    let mut d = dir.as_ref().to_path_buf();
                    // Ignore the first dir
                    d.extend(&f[1..]);
                    let _ = d.pop();
                    if !d.exists() {
                        create_dir_all(&d)?;
                    }

                    // checking if someone put /../ and tries to put something
                    // in the true root dir
                    let canon = d
                        .canonicalize()
                        .context("Couldn't canonicalize final extract path?")?;

                    if !canon.starts_with(&dir) {
                        bail!(
                            "This archive's got a path traversal? Final path goes out of {:?} resulting in {canon:?}",
                            dir.as_ref()
                        );
                    }

                    d.push(f_name);
                    log::info!("Extracting {d:?}");
                    let _ = e.unpack(&d)?;
                }
                Err(err) => {
                    log::error!("{err:?}");
                }
            }
        }

        Ok(())
    }
}

impl Default for MinecraftJavaVersion {
    fn default() -> Self {
        Self {
            // I truly don't know
            component: Arc::from("idk?"),
            // The newer ones will have 8, 17, or 21
            // But java 8 is the one's that used until 1.17 I believe
            major_version: 8,
        }
    }
}
