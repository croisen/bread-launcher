use std::env::var_os;
use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use fern::Dispatch;
use fern::colors::{Color, ColoredLevelConfig};
use fern::log_file;
use log::LevelFilter;
use reqwest::blocking::Client;

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

pub fn get_appdir() -> PathBuf {
    #[cfg(target_family = "windows")]
    let appdir = {
        let mut p = PathBuf::from(var_os("APPDATA").unwrap());
        p.push("Bread Launcher");

        p
    };

    #[cfg(target_family = "unix")]
    let appdir = {
        let mut p = PathBuf::from(var_os("HOME").unwrap());
        p.push(".local");
        p.push("share");
        p.push("breadlauncher");

        p
    };

    appdir
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
    let mut logger = Dispatch::new();
    let mut file = Dispatch::new()
        .format(|out, msg, rec| {
            out.finish(format_args!(
                "[{}] {} {} {}",
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
                "[{}] {} {} {}",
                Local::now().format("%d-%m-%Y %H:%M:%S"),
                COLORS.color(rec.level()),
                rec.target(),
                msg
            ));
        })
        .chain(std::io::stderr())
        .level(LevelFilter::Info);

    let name = Local::now().format("%Y-%m-%d.log");
    root.push("logs");
    root.push(name.to_string());
    let name = root.display().to_string();
    let _ = root.pop();
    create_dir_all(&root)?;
    let _ = root.pop();

    file = file.chain(log_file(name)?);
    logger = logger.chain(file);
    logger = logger.chain(stderr);
    logger.apply()?;

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
