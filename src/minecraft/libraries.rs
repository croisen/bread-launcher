use std::fs::{create_dir, File};
use std::io::copy as im_copy;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use zip::read::ZipArchive;

use crate::minecraft::MinecraftRule;
use crate::utils;

/**
 * Returns false if the name contains an arch name that doesn't match
 * the current platform's
 *
 * Though if it doesn't have any
 * it will probably be cleaned out by the OS rule after
 * so the macro always returns true if it doesn't find a match
 * big probably there tho
 */
macro_rules! check_arch {
    ($x: expr) => {{
        let z = {
            if $x.contains("x86") {
                "x86"
            } else if $x.contains("x64") {
                "x86_64"
            } else if $x.contains("arm") {
                "arm"
            } else if $x.contains("aarch_64") || $x.contains("aarch64") {
                // tf who else does aarch_64???
                "aarch64"
            } else if $x.contains("m68k") {
                "m68k"
            } else if $x.contains("mips") {
                "mips"
            } else if $x.contains("mips32r6") {
                "mips32r6"
            } else if $x.contains("mips64") {
                "mips64"
            } else if $x.contains("mips64r6") {
                "mips64r6"
            } else if $x.contains("csky") {
                "csky"
            } else if $x.contains("powerpc") {
                "powerpc"
            } else if $x.contains("powerpc64") {
                "powerpc64"
            } else if $x.contains("riscv32") {
                "riscv32"
            } else if $x.contains("riscv64") {
                "riscv64"
            } else if $x.contains("s390x") {
                "s390x"
            } else if $x.contains("sparc") {
                "sparc"
            } else if $x.contains("sparc64") {
                "sparc64"
            } else if $x.contains("hexagon") {
                "hexagon"
            } else if $x.contains("loongarch64") {
                "loongarch64"
            } else {
                std::env::consts::ARCH
            }
        };

        z == std::env::consts::ARCH
    }};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MinecraftLibArtifact {
    pub path: Arc<str>,
    pub sha1: Arc<str>,
    pub size: usize,
    pub url: Arc<str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLibClassifiers {
    #[serde(rename = "natives-linux")]
    natives_linux: Option<MinecraftLibArtifact>,
    #[serde(rename = "natives-windows")]
    natives_windows: Option<MinecraftLibArtifact>,
    #[serde(rename = "natives-windows-32")]
    natives_windows_32: Option<MinecraftLibArtifact>,
    #[serde(rename = "natives-windows-64")]
    natives_windows_64: Option<MinecraftLibArtifact>,
    #[serde(rename = "natives-osx")]
    natives_osx: Option<MinecraftLibArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLibDownload {
    artifact: Option<MinecraftLibArtifact>,
    classifiers: Option<MinecraftLibClassifiers>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftLibrary {
    downloads: MinecraftLibDownload,
    name: Arc<str>,
    rules: Option<Arc<Vec<MinecraftRule>>>,
}

impl MinecraftLibrary {
    pub async fn download(
        &self,
        cl: &Client,
        cache_dir: impl AsRef<Path>,
    ) -> Result<Option<PathBuf>> {
        if let Some(mla) = self.get_artifact() {
            if let Some(rules) = &self.rules {
                for rule in rules.iter() {
                    if !rule.is_needed() {
                        return Ok(None);
                    }
                }
            }

            let mut ld = cache_dir.as_ref().join("libraries");
            let v = mla.path.split("/").collect::<Vec<&str>>();
            let file = v.last().unwrap();
            ld.extend(v.iter());
            let _ = ld.pop();
            utils::download::download_with_sha(
                cl,
                &ld,
                file,
                &mla.url.clone(),
                &mla.sha1.clone(),
                true,
                1,
            )
            .await?;

            ld.push(file);
            if !self.extract_native_libs(mla, &ld, cache_dir.as_ref())? {
                return Ok(Some(ld));
            }
        }

        if let Some(nat) = self.get_native() {
            let mut ld = cache_dir.as_ref().join("libraries");
            let v = nat.path.split("/").collect::<Vec<&str>>();
            let file = v.last().unwrap();
            ld.extend(v.iter());
            let _ = ld.pop();
            utils::download::download_with_sha(
                cl,
                &ld,
                file,
                &nat.url.clone(),
                &nat.sha1.clone(),
                true,
                1,
            )
            .await?;

            ld.push(file);
            if !self.extract_native_libs(nat, &ld, cache_dir.as_ref())? {
                return Ok(Some(ld));
            }
        }

        Ok(None)
    }

    fn extract_native_libs(
        &self,
        mla: &MinecraftLibArtifact,
        jar: impl AsRef<Path>,
        cache_dir: impl AsRef<Path>,
    ) -> Result<bool> {
        if !mla.path.as_ref().contains("natives") {
            return Ok(false);
        }

        let mut l = cache_dir.as_ref().join("natives");
        if !l.is_dir() {
            create_dir(&l).context(format!("Was creating dir {l:?}"))?;
        }

        let f = File::open(jar.as_ref()).context(format!("Was opening file {:?}", jar.as_ref()))?;
        let mut z = ZipArchive::new(f)?;
        for i in 0..z.len() {
            let mut zf = z.by_index(i)?;
            if zf.is_dir() {
                continue;
            }

            if let Some(name) = zf.enclosed_name() {
                let fname = name
                    .components()
                    .next_back()
                    .unwrap()
                    .as_os_str()
                    .to_string_lossy();
                if fname.contains("MANIFEST") {
                    continue;
                }

                l.push(fname.as_ref());
                if l.exists() {
                    let _ = l.pop();
                    continue;
                }

                let mut ef = File::create(&l).context(format!(
                    "Full path to extract native lib {l:#?} doesn't exist?"
                ))?;

                im_copy(&mut zf, &mut ef)?;
                let _ = l.pop();
            }
        }

        Ok(true)
    }

    fn get_artifact(&self) -> Option<&MinecraftLibArtifact> {
        if let Some(mla) = &self.downloads.artifact {
            if check_arch!(self.name.clone().as_ref()) {
                return Some(mla);
            }
        };

        None
    }

    fn get_native(&self) -> Option<&MinecraftLibArtifact> {
        if let Some(cl) = &self.downloads.classifiers {
            if !check_arch!(self.name.clone().as_ref()) {
                return None;
            }

            #[cfg(target_os = "linux")]
            return cl.natives_linux.as_ref();

            #[cfg(target_os = "windows")]
            if let Some(nw) = &cl.natives_windows {
                return Some(nw);
            } else {
                #[cfg(target_arch = "x86")]
                return cl.natives_windows_32.as_ref();

                #[cfg(target_arch = "x86_64")]
                return cl.natives_windows_64.as_ref();
            }

            #[cfg(target_os = "macos")]
            return cl.natives_osx.as_ref();
        };

        None
    }
}
