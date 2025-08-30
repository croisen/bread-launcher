use std::env::var_os;
use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use fern::{Dispatch, log_file};
use log::LevelFilter;
use reqwest::blocking::Client;

pub static UNGROUPED_NAME: &str = "Venator A Mi Sumo Vela Mala";
pub static FULLNAME: &str = concat!("bread-launcher-v", env!("CARGO_PKG_VERSION"));
pub static VERSION: &str = env!("CARGO_PKG_VERSION");
pub static OAUTH_CLIENT_ID: &str = "I don't hab";

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
