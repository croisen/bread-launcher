#![allow(dead_code, unused_variables)]

use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;
use std::thread;

use anyhow::Result;
use reqwest::ClientBuilder;
use serde::Deserialize;
use serde_json::Deserializer;

mod assets;
mod logs;
mod minecraft;
mod utils;

fn main() {
    let r = logs::init_logs_and_appdir();
    if let Err(e) = &r {
        eprintln!("Failed to init logs: {e}");
        return;
    }

    let approot = r.unwrap();
    let r = tokio::runtime::Builder::new_multi_thread()
        .thread_name("bread-launcher-main")
        .enable_all()
        .build();
    if let Err(e) = &r {
        log::error!("Yabe: {e}");
        return;
    }

    let h = thread::spawn(move || {
        let r = r.unwrap();
        if let Err(e) = r.block_on(start_async(&approot)) {
            log::error!("Yabe: {e}");
        }
    });

    let _ = h.join();
}

async fn start_async(appdir: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let p = appdir.as_ref().join("cache/1.21.5/client.json");
    let f = OpenOptions::new().read(true).open(p)?;

    let m = minecraft::Minecraft::deserialize(&mut Deserializer::from_reader(f))?;
    println!("{m:#?}");
    Ok(())
}
