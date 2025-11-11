use std::env::consts::ARCH as CURRENT_ARCH;
use std::fs::{File, create_dir_all, write};
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use zip::read::ZipArchive;

use crate::init::get_libdir;
use crate::loaders::minecraft::MinecraftRule;
use crate::utils::download::download_with_sha1;

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
    pub path: String,
    pub sha1: String,
    pub size: usize,
    pub url: String,
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
    name: String,
    rules: Option<Vec<MinecraftRule>>,
}

impl MinecraftLibrary {
    /// Returns none if it's blocked by a rule, url is empty, or is just not
    /// supported
    pub fn get_path(&self) -> Option<PathBuf> {
        if !self.is_needed() {
            return None;
        }

        if let Some(cla) = &self.downloads.classifiers {
            let nat = if cfg!(target_os = "linux") {
                cla.natives_linux.as_ref()
            } else if cfg!(target_os = "macos") {
                cla.natives_osx.as_ref()
            } else if cfg!(target_os = "windows") {
                if cla.natives_windows.is_some() {
                    cla.natives_windows.as_ref()
                } else if cfg!(target_arch = "x86") {
                    cla.natives_windows_32.as_ref()
                } else if cfg!(target_arch = "x86_64") {
                    cla.natives_windows_64.as_ref()
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(nat) = nat {
                let mut ld = get_libdir();
                ld.extend(nat.path.split("/"));
                return Some(ld);
            }
        }

        if let Some(mla) = &self.downloads.artifact {
            let mut ld = get_libdir();
            ld.extend(mla.path.split("/"));
            return Some(ld);
        }

        None
    }

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

    pub fn download_library(&self, cl: Client, instance_dir: impl AsRef<Path>) -> Result<()> {
        if !self.is_needed() {
            return Ok(());
        }

        self.download_classified(cl.clone(), &instance_dir)?;
        self.download_artifact(cl, &instance_dir)?;

        Ok(())
    }

    pub fn extract_if_native_lib(&self, instance_dir: impl AsRef<Path>) -> Result<()> {
        if !self.is_needed() {
            return Ok(());
        }

        if let Some(mla) = &self.downloads.artifact {
            let mut ld = get_libdir();
            ld.extend(mla.path.split("/"));
            let _ = self
                .extract_native_libs(mla, &ld, &instance_dir)
                .context("Was extracting native libs")?;
        }

        if self.downloads.classifiers.is_none() {
            return Ok(());
        }

        let cla = self.downloads.classifiers.as_ref().unwrap();
        let nat = if cfg!(target_os = "linux") {
            cla.natives_linux.as_ref()
        } else if cfg!(target_os = "macos") {
            cla.natives_osx.as_ref()
        } else if cfg!(target_os = "windows") {
            if cla.natives_windows.is_some() {
                cla.natives_windows.as_ref()
            } else if cfg!(target_arch = "x86") {
                cla.natives_windows_32.as_ref()
            } else if cfg!(target_arch = "x86_64") {
                cla.natives_windows_64.as_ref()
            } else {
                None
            }
        } else {
            None
        };

        if nat.is_none() {
            return Ok(());
        }

        let nat = nat.unwrap();
        // contains an architecture in the name but doesn't match the current machine's
        // though this only happens in the older versions I believe
        if let Some(arch) = check_arch!(nat.path)
            && arch != CURRENT_ARCH
        {
            return Ok(());
        }

        let mut ld = get_libdir();
        ld.extend(nat.path.split("/"));
        let _ = self
            .extract_native_libs(nat, &ld, instance_dir)
            .context("Was extracting native libs")?;

        Ok(())
    }

    fn extract_native_libs(
        &self,
        mla: &MinecraftLibArtifact,
        jar: impl AsRef<Path>,
        cache_dir: impl AsRef<Path>,
    ) -> Result<bool> {
        if !mla.path.contains("natives") {
            return Ok(false);
        }

        let mut l = cache_dir.as_ref().join("natives");
        if !l.is_dir() {
            create_dir_all(&l).context(format!("Was creating dir {l:?}"))?;
        }

        let f = File::open(&jar).context(format!("Was opening file {:?}", jar.as_ref()))?;
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
                    .display()
                    .to_string();

                if fname.contains("MANIFEST") {
                    continue;
                }

                l.push(fname);
                if l.exists() {
                    let _ = l.pop();
                    continue;
                }

                let mut buf = vec![];
                zf.read_to_end(&mut buf)?;
                write(&l, buf.as_slice()).context(format!(
                    "Full path to extract native lib {l:#?} doesn't exist?"
                ))?;

                let _ = l.pop();
            }
        }

        Ok(true)
    }

    fn download_artifact(&self, cl: Client, instance_dir: impl AsRef<Path>) -> Result<()> {
        if self.downloads.artifact.is_none() {
            return Ok(());
        }

        if let Some(mla) = &self.downloads.artifact {
            let mut ld = get_libdir();
            ld.extend(mla.path.split("/"));
            let file = ld.file_name().unwrap().display().to_string();
            let _ = ld.pop();
            download_with_sha1(&cl, &ld, &file, &mla.url, &mla.sha1, 1)?;

            ld.push(&file);
            let _ = self
                .extract_native_libs(mla, &ld, instance_dir)
                .context("Was extracting native libs")?;
        }

        Ok(())
    }

    fn download_classified(&self, cl: Client, instance_dir: impl AsRef<Path>) -> Result<()> {
        if self.downloads.classifiers.is_none() {
            return Ok(());
        }

        let cla = self.downloads.classifiers.as_ref().unwrap();
        let nat = if cfg!(target_os = "linux") {
            cla.natives_linux.as_ref()
        } else if cfg!(target_os = "macos") {
            cla.natives_osx.as_ref()
        } else if cfg!(target_os = "windows") {
            if cla.natives_windows.is_some() {
                cla.natives_windows.as_ref()
            } else if cfg!(target_arch = "x86") {
                cla.natives_windows_32.as_ref()
            } else if cfg!(target_arch = "x86_64") {
                cla.natives_windows_64.as_ref()
            } else {
                None
            }
        } else {
            None
        };

        if nat.is_none() {
            return Ok(());
        }

        let nat = nat.unwrap();
        // contains an architecture in the name but doesn't match the current machine's
        // though this only happens in the older versions I believe
        if let Some(arch) = check_arch!(nat.path)
            && arch != CURRENT_ARCH
        {
            return Ok(());
        }

        let mut ld = get_libdir();
        ld.extend(nat.path.split("/"));
        let file = ld.file_name().unwrap().display().to_string();
        let _ = ld.pop();
        download_with_sha1(&cl, &ld, &file, &nat.url, &nat.sha1, 1)?;

        ld.push(&file);
        let _ = self
            .extract_native_libs(nat, &ld, instance_dir)
            .context("Was extracting native libs")?;

        Ok(())
    }
}
