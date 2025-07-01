use std::env::consts::ARCH as CURRENT_ARCH;
use std::fs::{File, create_dir};
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
        if $x.contains("x86") {
            Some("x86")
        } else if $x.contains("x64") {
            Some("x86_64")
        } else if $x.contains("arm") {
            Some("arm")
        } else if $x.contains("aarch_64") || $x.contains("aarch64") {
            // tf who else does aarch_64???
            Some("aarch64")
        } else if $x.contains("m68k") {
            Some("m68k")
        } else if $x.contains("mips") {
            Some("mips")
        } else if $x.contains("mips32r6") {
            Some("mips32r6")
        } else if $x.contains("mips64") {
            Some("mips64")
        } else if $x.contains("mips64r6") {
            Some("mips64r6")
        } else if $x.contains("csky") {
            Some("csky")
        } else if $x.contains("powerpc") {
            Some("powerpc")
        } else if $x.contains("powerpc64") {
            Some("powerpc64")
        } else if $x.contains("riscv32") {
            Some("riscv32")
        } else if $x.contains("riscv64") {
            Some("riscv64")
        } else if $x.contains("s390x") {
            Some("s390x")
        } else if $x.contains("sparc") {
            Some("sparc")
        } else if $x.contains("sparc64") {
            Some("sparc64")
        } else if $x.contains("hexagon") {
            Some("hexagon")
        } else if $x.contains("loongarch64") {
            Some("loongarch64")
        } else {
            None
        }
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
    pub fn is_needed(&self) -> bool {
        if let Some(rules) = &self.rules {
            for rule in rules.iter() {
                if !rule.is_needed() {
                    return false;
                }
            }
        }

        true
    }

    /// Returns none if it's native or blocked by a rule
    pub fn get_path(&self, cache_dir: impl AsRef<Path>) -> Option<PathBuf> {
        if !self.is_needed() {
            return None;
        }

        if let Some(cla) = &self.downloads.classifiers {
            #[cfg(target_os = "linux")]
            let nat = cla.natives_linux.as_ref();
            #[cfg(target_os = "macos")]
            let nat = cla.natives_osx.as_ref();

            #[cfg(target_os = "windows")]
            let nat = {
                if cla.natives_windows.is_some() {
                    cla.natives_windows.as_ref()
                } else {
                    #[cfg(target_arch = "x86")]
                    {
                        cla.natives_windows_32.as_ref()
                    }

                    #[cfg(target_arch = "x86_64")]
                    {
                        cla.natives_windows_64.as_ref()
                    }
                }
            };

            if let Some(nat) = nat {
                let mut ld = cache_dir.as_ref().join("libraries");
                ld.extend(nat.path.split("/"));

                if nat.path.contains("natives") {
                    return Some(ld);
                } else {
                    return None;
                }
            }
        }

        if let Some(mla) = &self.downloads.artifact {
            let mut ld = cache_dir.as_ref().join("libraries");
            ld.extend(mla.path.split("/"));

            if mla.path.contains("natives") {
                return Some(ld);
            } else {
                return None;
            }
        }

        None
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

    /// Returns an optional lib path if it's not a native lib
    pub async fn download_library(
        &self,
        cl: &Client,
        cache_dir: impl AsRef<Path>,
    ) -> Result<Option<PathBuf>> {
        if !self.is_needed() {
            return Ok(None);
        }

        let art = self.download_artifact(cl, &cache_dir).await?;
        let cla = self.download_classified(cl, &cache_dir).await?;
        let ret = if cla.is_some() { cla } else { art };

        Ok(ret)
    }

    async fn download_artifact(
        &self,
        cl: &Client,
        cache_dir: impl AsRef<Path>,
    ) -> Result<Option<PathBuf>> {
        if self.downloads.artifact.is_none() {
            return Ok(None);
        }

        if let Some(mla) = &self.downloads.artifact {
            let mut ld = cache_dir.as_ref().join("libraries");
            let v = mla.path.split("/").collect::<Vec<&str>>();
            let file = v.last().unwrap();
            ld.extend(v.iter());
            let _ = ld.pop();
            utils::download::download_with_sha(cl, &ld, file, &mla.url, &mla.sha1, true, 1).await?;
            ld.push(file);

            let is_native = self.extract_native_libs(mla, &ld, cache_dir)?;
            if is_native { Ok(Some(ld)) } else { Ok(None) }
        } else {
            Ok(None)
        }
    }

    async fn download_classified(
        &self,
        cl: &Client,
        cache_dir: impl AsRef<Path>,
    ) -> Result<Option<PathBuf>> {
        if self.downloads.classifiers.is_none() {
            return Ok(None);
        }

        let cla = self.downloads.classifiers.as_ref().unwrap();

        #[cfg(target_os = "linux")]
        let nat = cla.natives_linux.as_ref();
        #[cfg(target_os = "macos")]
        let nat = cla.natives_osx.as_ref();

        #[cfg(target_os = "windows")]
        let nat = {
            if cla.natives_windows.is_some() {
                cla.natives_windows.as_ref()
            } else {
                #[cfg(target_arch = "x86")]
                {
                    cla.natives_windows_32.as_ref()
                }

                #[cfg(target_arch = "x86_64")]
                {
                    cla.natives_windows_64.as_ref()
                }
            }
        };

        if nat.is_none() {
            return Ok(None);
        }

        let nat = nat.unwrap();
        // contains an architecture in the name but doesn't match the current machine's
        // though this only happens in the older versions I believe
        if let Some(arch) = check_arch!(nat.path) {
            if arch != CURRENT_ARCH {
                return Ok(None);
            }
        }

        let mut ld = cache_dir.as_ref().join("libraries");
        let v = nat.path.split("/").collect::<Vec<&str>>();
        let file = v.last().unwrap();
        ld.extend(v.iter());
        let _ = ld.pop();
        utils::download::download_with_sha(cl, &ld, file, &nat.url, &nat.sha1, true, 1).await?;
        ld.push(file);

        let is_native = self.extract_native_libs(nat, &ld, cache_dir)?;
        if is_native { Ok(Some(ld)) } else { Ok(None) }
    }
}
