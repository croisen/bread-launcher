use std::env::var_os;
use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use fern::{Dispatch, log_file};
use log::LevelFilter;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

pub static UNGROUPED_NAME: &str = "Venator A Mi Sumo Vela Mala";
pub static FULLNAME: &str = concat!("bread-launcher-v", env!("CARGO_PKG_VERSION"));
pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub static OAUTH_CLIENT_ID: &str = "I don't hab";

// Remotes
pub static R_MINECRAFT_VER: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
pub static R_MINECRAFT_MVN: &str = "https://libraries.minecraft.net";
pub static R_FORGE_VER: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
pub static R_FORGE_REC: &str =
    "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";
pub static R_FABRIC_VER: &str = "https://meta.fabricmc.net/v2/versions";
pub static R_LITE_VER: &str = "http://dl.liteloader.com/versions/versions.json";
pub static R_QUILT_VER: &str = "https://meta.quiltmc.org/v3/versions";

// ${appdir}/loaders/*.json
pub static L_MINECRAFT_VER: &str = "minecraft_versions.json";
pub static L_FORGE_VER: &str = "forge_versions.json";
pub static L_FORGE_REC: &str = "forge_recommend.json";
pub static L_FABRIC_VER: &str = "fabric_versions.json";
pub static L_LITE_VER: &str = "liteloader_versions.json";
pub static L_QUILT_VER: &str = "quilt_versions.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub jvm_ram: usize, // in MB
}

impl Default for Settings {
    fn default() -> Self {
        Self { jvm_ram: 2048 }
    }
}

pub fn get_appdir() -> PathBuf {
    if cfg!(windows) {
        let mut main = var_os("APPDATA").unwrap();
        main.push("\\Bread Launcher");
        PathBuf::from(main)
    } else {
        let mut main = var_os("HOME").unwrap();
        main.push("/.local/share/bread-launcher");
        PathBuf::from(main)
    }
}

pub fn get_instancedir() -> PathBuf {
    let mut ad = get_appdir();
    ad.push("instances");

    ad
}

pub fn get_javadir() -> PathBuf {
    let mut ad = get_appdir();
    ad.push("java");

    ad
}

pub fn get_tempdir() -> PathBuf {
    let mut ad = get_appdir();
    ad.push("temp");

    ad
}

pub fn get_cachedir() -> PathBuf {
    let mut ad = get_appdir();
    ad.push("cache");

    ad
}

pub fn get_assetsdir() -> PathBuf {
    let mut cd = get_cachedir();
    cd.push("assets");

    cd
}

pub fn get_libdir() -> PathBuf {
    let mut cd = get_cachedir();
    cd.push("libraries");

    cd
}

pub fn get_versiondir() -> PathBuf {
    let mut cd = get_cachedir();
    cd.push("versions");

    cd
}

/// ${appdir}/loaders/minecraft_versions.json
pub fn get_vanilla_loader() -> PathBuf {
    let mut ad = get_appdir();
    ad.extend(["loaders", L_MINECRAFT_VER]);

    ad
}

/// ${appdir}/cache/versions/version.json
pub fn get_vanilla_path(mc_ver: impl AsRef<str>) -> PathBuf {
    let mut vd = get_versiondir();
    vd.push(format!("{}.json", mc_ver.as_ref()));

    vd
}

/// ${appdir}/loaders/{forge_vers.json,forge_recs.json}
pub fn get_forge_loader() -> (PathBuf, PathBuf) {
    let mut ad1 = get_appdir();
    let mut ad2 = get_appdir();
    ad1.extend(["loaders", L_FORGE_VER]);
    ad2.extend(["loaders", L_FORGE_REC]);

    (ad1, ad2)
}

/// ${appdir}/cache/versions/version.json
pub fn get_forge_path(forge_ver: impl AsRef<str>) -> PathBuf {
    let mut vd = get_versiondir();
    vd.push(format!("{}.json", forge_ver.as_ref()));

    vd
}

pub fn init_logs() -> Result<()> {
    let mut root = get_appdir();
    let mut file = Dispatch::new()
        .format(|out, msg, rec| {
            out.finish(format_args!(
                "[{}] {:<5} {} {}",
                Local::now().format("%d-%m-%Y %H:%M:%S"),
                rec.level(),
                rec.target(),
                msg
            ));
        })
        .level(LevelFilter::Debug);

    let stderr = Dispatch::new()
        .format(|out, msg, rec| {
            out.finish(format_args!(
                "[{}] {:<5} {} {}",
                Local::now().format("%d-%m-%Y %H:%M:%S"),
                COLORS.color(rec.level()),
                rec.target(),
                msg
            ));
        })
        .chain(std::io::stderr())
        .level(LevelFilter::Info);

    root.push("logs");
    create_dir_all(&root)?;
    root.push(Local::now().format("%Y-%m-%d.log").to_string());
    file = file.chain(log_file(root.display().to_string())?);
    Dispatch::new().chain(file).chain(stderr).apply()?;

    Ok(())
}

pub fn init_reqwest() -> Result<Client> {
    let user_agent = if cfg!(windows) {
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
    } else if cfg!(unix) {
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36"
    } else {
        FULLNAME
    };

    let c = Client::builder()
        .user_agent(user_agent)
        .pool_idle_timeout(None)
        .use_rustls_tls()
        .https_only(true)
        .build()?;

    Ok(c)
}

static COLORS: ColoredLevelConfig = ColoredLevelConfig {
    error: Color::TrueColor {
        r: 255,
        g: 20,
        b: 20,
    },
    warn: Color::TrueColor {
        r: 255,
        g: 255,
        b: 20,
    },
    info: Color::TrueColor {
        r: 20,
        g: 255,
        b: 20,
    },
    debug: Color::TrueColor {
        r: 255,
        g: 20,
        b: 255,
    },
    trace: Color::TrueColor {
        r: 255,
        g: 255,
        b: 255,
    },
};
