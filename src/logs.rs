use std::env::var_os;
use std::error::Error;
use std::fs::create_dir_all;
use std::path::PathBuf;

use chrono::Local;
use fern::colors::Color;
use fern::colors::ColoredLevelConfig;
use fern::log_file;
use fern::Dispatch;
use log::LevelFilter;

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

pub fn init_logs_and_appdir() -> Result<PathBuf, Box<dyn Error>> {
    fern::colors::Color::TrueColor {
        r: 100,
        g: 200,
        b: 200,
    };

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
        .level(LevelFilter::Info);
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

    #[cfg(target_family = "windows")]
    let mut root = {
        let mut p = PathBuf::from(var_os("APPDATA").unwrap());
        p.push("Bread_Launcher");

        p
    };
    #[cfg(target_family = "unix")]
    let mut root = {
        let mut p = PathBuf::from(var_os("HOME").unwrap());
        p.push(".local");
        p.push("share");
        p.push("bread_launcher");

        p
    };

    let name = Local::now().format("%Y-%m-%d.log");
    root.push("logs");
    root.push(name.to_string());
    let name = root.to_str().unwrap().to_string();
    let _ = root.pop();
    create_dir_all(&root)?;
    let _ = root.pop();

    file = file.chain(log_file(name)?);
    logger = logger.chain(file);
    logger = logger.chain(stderr);
    logger.apply()?;

    Ok(root)
}
