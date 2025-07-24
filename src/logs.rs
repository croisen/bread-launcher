use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use chrono::Local;
use fern::Dispatch;
use fern::colors::Color;
use fern::colors::ColoredLevelConfig;
use fern::log_file;
use log::LevelFilter;

use crate::init::init_appdir;

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

pub fn init_logs_and_appdir() -> Result<PathBuf> {
    let mut root = init_appdir()?;
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
    let name = root
        .to_str()
        .ok_or(anyhow!("Could not convert path {root:?} to string"))?
        .to_string();

    let _ = root.pop();
    create_dir_all(&root)?;
    let _ = root.pop();

    file = file.chain(log_file(name)?);
    logger = logger.chain(file);
    logger = logger.chain(stderr);
    logger.apply()?;

    Ok(root)
}
