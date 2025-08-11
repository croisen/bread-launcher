use std::env::var;
use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use fern::{Dispatch, log_file};
use log::LevelFilter;
use reqwest::Client;

pub static UNGROUPED_NAME: &str = "Venator A Mi Sumo Vela Mala";
pub static FULLNAME: &str = concat!("bread-launcher-v", env!("CARGO_PKG_VERSION"));
pub static VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn get_appdir() -> PathBuf {
    if cfg!(target_family = "windows") {
        PathBuf::from(format!("{}\\Bread Launcher", var("APPDATA").unwrap()))
    } else {
        PathBuf::from(format!(
            "{}/.local/share/breadlauncher",
            var("HOME").unwrap()
        ))
    }
}

pub fn get_instancedir() -> PathBuf {
    get_appdir().join("instances")
}

pub fn get_javadir() -> PathBuf {
    get_appdir().join("java")
}

pub fn get_tempdir() -> PathBuf {
    get_appdir().join("temp")
}

pub fn get_cachedir() -> PathBuf {
    get_appdir().join("cache")
}

pub fn get_assetsdir() -> PathBuf {
    get_cachedir().join("assets")
}

pub fn get_libdir() -> PathBuf {
    get_cachedir().join("libraries")
}

pub fn get_versiondir() -> PathBuf {
    get_cachedir().join("versions")
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
    #[cfg(target_family = "windows")]
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";
    #[cfg(target_family = "unix")]
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

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
