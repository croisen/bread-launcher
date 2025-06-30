#![allow(dead_code, unused_variables)]
#![feature(pattern)]

use std::sync::mpsc::channel;
use std::thread::spawn;

use anyhow::{Context, Result};
use egui_extras::install_image_loaders;
use reqwest::Client;
use tokio::runtime::Handle;

mod app;
mod assets;
mod instance;
mod logs;
mod minecraft;
mod utils;

fn main() -> Result<()> {
    let appdir = logs::init_logs_and_appdir()?;
    let cl = Client::builder()
        .user_agent(format!("bread-launcher-v-{}", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .use_rustls_tls()
        .pool_max_idle_per_host(0)
        .build()
        .context("Could not build reqwest client")?;

    let (tx, rx) = channel::<()>();
    let (txh, rxh) = channel::<Handle>();
    let t = spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name("bread-launcher-async-main")
            .enable_all()
            .build()
            .context("Could not build tokio runtime")
            .unwrap();

        let _ = txh.send(runtime.handle().clone());
        runtime.block_on(app::event_loop(rx));
    });

    let handle = rxh.recv()?;
    let app = app::BreadLauncher::new(cl, &appdir, handle)?;
    let _ = eframe::run_native(
        "Bread Launcher",
        eframe::NativeOptions {
            persistence_path: Some(appdir.join("egui.ron")),
            viewport: egui::ViewportBuilder::default(),
            vsync: true,
            ..Default::default()
        },
        Box::new(move |cc| {
            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );

    let _ = tx.send(());
    let _ = t.join();
    Ok(())
}
