use std::env::consts;
use std::ffi::OsStr;
use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use std::str::pattern::Pattern;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::fs::remove_file as tk_remove_file;

#[cfg(target_family = "unix")]
use flate2::read::GzDecoder;
use tar::Archive;

#[cfg(target_family = "windows")]
use zip::read::{root_dir_common_filter, ZipArchive};

use crate::utils;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftJavaVersion {
    component: Arc<str>,
    #[serde(rename = "majorVersion")]
    major_version: usize,
}

impl MinecraftJavaVersion {
    pub async fn download(&self, cl: &Client, root_dir: impl AsRef<Path>) -> Result<PathBuf> {
        #[cfg(target_arch = "x86_64")]
        let arch = "x64";
        #[cfg(not(target_arch = "x86_64"))]
        let arch = consts::ARCH;

        // We're using Temurin
        let jre: Arc<str> = Arc::from(format!(
            "https://api.adoptium.net/v3/binary/latest/{}/ga/{}/{}/jre/hotspot/normal/eclipse?project=jdk",
            self.major_version,
            consts::OS,
            arch,
        ));

        let mut j = root_dir.as_ref().join("java");
        j.push(format!("{:0>2}", self.major_version));
        j.push("bin");
        if !j.is_dir() {
            let _ = j.pop();
            let mut t = root_dir.as_ref().join("temp");
            // Not exactly a zip in all platforms but I'm feeling lazy
            utils::download::download(cl, &t, "temurin.zip", &jre).await?;
            t.push("temurin.zip");

            let f = File::open(&t).context("Could not open downloaded jre archive?")?;
            #[cfg(target_family = "windows")]
            ZipArchive::new(f)?.extract_unwrapped_root_dir(&j, root_dir_common_filter)?;
            #[cfg(target_family = "unix")]
            self.extract_unwrapped_root_dir(f, &j)?;

            j.push("bin");
            tk_remove_file(&t).await?;
        }

        #[cfg(target_family = "unix")]
        j.push("java");
        #[cfg(target_family = "windows")]
        j.push("javaw.exe");

        Ok(j)
    }

    #[cfg(target_family = "unix")]
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
                    let f_name = f.last().unwrap();

                    let mut d = dir.as_ref().to_path_buf();
                    // Ignore the first dir
                    d.extend(&f[1..]);
                    let _ = d.pop();
                    create_dir_all(&d)?;
                    d.canonicalize()
                        .context("Couldn't canonicalize final extract path?")?;

                    // checking if someone put /../ and tries to put something
                    // in the true root dir
                    let d_pref = dir.as_ref().to_str().unwrap();
                    let d_res = d.to_str().unwrap();
                    if !d_pref.is_prefix_of(d_res) {
                        return Err(anyhow!(
                            "This archive's got a path traversal? Final path goes out of\n{}\nresulting in\n{}",
                            d_pref, d_res
                        ));
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
