use std::fs::read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{Context, Result, anyhow};
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use uuid::Builder as UB;
use uuid::Version;

mod arguments;
mod assets;
mod downloads;
mod java_version;
mod libraries;
mod organized;
mod rules;
mod version_manifest;

pub use arguments::MinecraftArgument;
pub use assets::MinecraftAsset;
pub use downloads::MinecraftDownload;
pub use java_version::MinecraftJavaVersion;
pub use libraries::MinecraftLibrary;
pub use organized::MVOrganized;
pub use rules::MinecraftRule;
pub use version_manifest::MinecraftVersionManifest;

use crate::utils::fs;

/// Some attr are public so the application GUI itself can do the downloading
/// and gathering the arguments?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minecraft {
    pub arguments: Option<Arc<MinecraftArgument>>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<Arc<arguments::Argument>>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Arc<MinecraftAsset>,
    pub downloads: Arc<MinecraftDownload>,
    #[serde(default, rename = "javaVersion")]
    pub java_version: Arc<MinecraftJavaVersion>,
    pub libraries: Vec<Arc<MinecraftLibrary>>,

    pub id: Arc<str>,
    #[serde(rename = "mainClass")]
    main_class: Arc<str>,
    #[serde(rename = "minimumLauncherVersion")]
    minimum_launcher_version: usize,
    #[serde(rename = "releaseTime")]
    pub release_time: Arc<str>,
    time: Arc<str>,
    #[serde(rename = "type")]
    release_type: Arc<str>,

    #[serde(skip_deserializing)]
    pub appdir: Arc<PathBuf>,
    #[serde(skip_deserializing)]
    pub cache_dir: Arc<PathBuf>,
}

impl Minecraft {
    pub fn new(cache_dir: impl AsRef<Path>) -> Result<Self> {
        let mut ad = cache_dir.as_ref().to_path_buf();
        let mut json = cache_dir.as_ref().join("client.json");
        let f = read(&json).context(format!("Failed to read {json:?}"))?;
        let mut de = Deserializer::from_slice(f.as_ref());
        let mut m = Self::deserialize(&mut de).context(format!(
            "Failed to desrialize minecraft client data from {json:?}"
        ))?;

        log::info!("MC Version:   {}", m.id.as_ref());
        log::info!("Java Version: {:?}", m.java_version.as_ref());
        let _ = ad.pop();
        let _ = ad.pop();
        let _ = json.pop();

        m.appdir = Arc::new(ad);
        m.cache_dir = Arc::new(json);
        Ok(m)
    }

    //    pub async fn download(&self, cl: &Client) -> Result<(Vec<String>, Vec<String>)> {
    //        let mut jvm_args = vec![];
    //        let mut mc_args = vec![];
    //
    //        log::info!("Checking client main files");
    //        self.downloads.download(cl, self.cache_dir.as_ref()).await?;
    //        log::info!("Checking client assets");
    //        let asset_index = self
    //            .asset_index
    //            .download(cl, self.cache_dir.as_ref())
    //            .await?;
    //        log::info!("Checking java runtime environment");
    //        let jre = self.java_version.download(cl, self.appdir.as_ref()).await?;
    //
    //        log::info!("JRE path: {jre:?}");
    //        log::info!("Checking client libraries");
    //
    //        jvm_args.push(jre.to_string_lossy().to_string());
    //        jvm_args.push("-Dminecraft.launcher.brand=bread-launcher".to_string());
    //        jvm_args.push(format!(
    //            "-Dminecraft.launcher.version={}",
    //            env!("CARGO_PKG_VERSION")
    //        ));
    //        jvm_args.push(format!(
    //            "-Djava.library.path={}",
    //            self.cache_dir.join("natives").to_string_lossy()
    //        ));
    //        jvm_args.push("-cp".to_string());
    //        mc_args.push("--assetIndex".to_string());
    //        mc_args.push(asset_index);
    //        let mut gd = self.cache_dir.join(".minecraft");
    //        mc_args.push("--gameDir".to_string());
    //        mc_args.push(gd.to_string_lossy().to_string());
    //        mc_args.push("--assetsDir".to_string());
    //        gd.push("assets");
    //        mc_args.push(gd.to_string_lossy().to_string());
    //
    //        let mut libs = vec![];
    //        libs.push(
    //            self.cache_dir
    //                .join("client.jar")
    //                .to_string_lossy()
    //                .to_string(),
    //        );
    //        for lib in &self.libraries {
    //            if let Some(l) = lib.download(cl, self.cache_dir.as_ref()).await? {
    //                libs.push(l.to_string_lossy().to_string());
    //            }
    //        }
    //
    //        jvm_args.push(libs.join(":"));
    //        Ok((jvm_args, mc_args))
    //    }

    async fn get_arguments(
        &self,
        cl: Client,
        ram: String,
        username: String,
        access_token: String,
        user_properties: String,
    ) -> Result<(String, Vec<String>, Vec<String>)> {
        let mut jre = self.appdir.join("java");
        jre.push(format!("{:0>2}", self.java_version.get_version()));
        jre.push("bin");
        #[cfg(target_family = "unix")]
        jre.push("java");
        #[cfg(target_family = "windows")]
        jre.push("javaw.exe");
        let jre = jre
            .to_str()
            .ok_or(anyhow!("Path is not valid unicode???"))?
            .to_string();

        let gd = self.cache_dir.join(".minecraft");
        let ad = gd.join("assets");

        let jvm_args = vec![
            format!("-Xms{ram}"),
            format!("-Xmx{ram}"),
            "-Dminecraft.launcher.brand=bread-launcher".to_string(),
            format!("-Dminecraft.launcher.version={}", env!("CARGO_PKG_VERSION")),
            format!(
                "-Djava.library.path={}",
                self.cache_dir.join("natives").to_string_lossy()
            ),
            "-cp".to_string(),
            // Gotta pop one off of the jvm_args if I plan to use forge or other
            // mod loaders to launch minecraft, or just make another one of this
            // function, or inline it
            self.main_class.as_ref().to_string(),
        ];

        let mc_args = vec![
            "--assetIndex".to_string(),
            self.asset_index.get_id().to_string(),
            "--gameDir".to_string(),
            gd.to_str()
                .ok_or(anyhow!("Path is not valid unicode???"))?
                .to_string(),
            "--assetDir".to_string(),
            ad.to_str()
                .ok_or(anyhow!("Path is not valid unicode???"))?
                .to_string(),
            "--username".to_string(),
            username,
            "--accessToken".to_string(),
            access_token,
            "--userProperties".to_string(),
            user_properties,
            "--version".to_string(),
            self.id.as_ref().to_string(),
        ];

        Ok((jre, jvm_args, mc_args))
    }

    pub async fn run(
        &self,
        cl: Client,
        ram: String,
        username: String,
        access_token: String,
        user_properties: String,
    ) -> Result<()> {
        let (jre, jvm_args, mc_args) = self
            .get_arguments(cl, ram, username, access_token, user_properties)
            .await?;

        let mut child = Command::new(jre)
            .current_dir(self.get_cache_dir().join(".minecraft"))
            .args(&jvm_args)
            .args(mc_args)
            .spawn()
            .context(format!(
                "Failed to start minecraft with jvm {}",
                jvm_args[0]
            ))?;

        let status = child.wait()?;
        log::info!("Run exit status: {:?}", status.code());

        Ok(())
    }

    pub fn new_insatance(&self) -> Result<Self> {
        log::info!("Creating new instance for MC ver {}", self.id.as_ref());

        let mut s = self.clone();
        let mut c = self.appdir.join("instances");
        let ts = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
        let mut rng = StdRng::try_from_os_rng()?;
        let mut rb: [u8; 10] = [0; 10];
        rng.fill_bytes(&mut rb);

        let u = UB::from_unix_timestamp_millis(ts.as_millis().try_into()?, &rb)
            .with_version(Version::SortRand)
            .into_uuid()
            .to_string();

        c.push(&u);

        s.cache_dir = Arc::new(c);
        fs::scopy(self.cache_dir.as_ref(), s.cache_dir.as_ref())?;
        log::info!(
            "New instance created in dir {:?} with MC ver {}",
            s.cache_dir,
            s.id.as_ref()
        );

        Ok(s)
    }

    pub fn get_cache_dir(&self) -> Arc<PathBuf> {
        self.cache_dir.clone()
    }
}
