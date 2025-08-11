use std::io::Cursor;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::thread::{available_parallelism, spawn};
use std::time::Duration;

use anyhow::Result;
use eframe::{NativeOptions, run_native};
use egui::{ColorImage, IconData, TextureOptions};
use egui_extras::install_image_loaders;
use image::ImageReader;
use tokio::runtime::Builder;
use tokio::time::sleep;

use crate::app::BreadLauncher;
use crate::assets::ICONS;
use crate::init::{get_appdir, init_logs};

pub fn launch() -> Result<()> {
    init_logs()?;

    let img = ImageReader::new(Cursor::new(ICONS[0].1))
        .with_guessed_format()?
        .decode()?;

    let size = [img.width() as _, img.height() as _];
    let buffer = img.to_rgba8();
    let pixels = buffer.as_flat_samples();
    let app_icon = Arc::new(IconData {
        rgba: pixels.to_vec().samples,
        width: size[0],
        height: size[1],
    });

    let icon = app_icon.clone();
    let opt = NativeOptions {
        persist_window: true,
        persistence_path: Some(get_appdir().join("save.ron")),
        vsync: true,
        window_builder: Some(Box::new(move |vb| vb.with_icon(icon))),
        viewport: egui::ViewportBuilder {
            icon: Some(app_icon),
            ..Default::default()
        },
        ..Default::default()
    };

    let (tx, rx) = channel::<()>();
    let rt = Builder::new_multi_thread()
        .worker_threads(available_parallelism()?.get())
        .max_blocking_threads(512)
        .thread_name("Bread Launcher (async rt)")
        .enable_all()
        .build()?;

    let _g = rt.enter();
    let rt_thread = spawn(move || {
        rt.block_on(async {
            loop {
                if rx.try_recv().is_ok() {
                    log::warn!("Async runtime is now stopping...");
                    break;
                }

                sleep(Duration::from_secs(1)).await;
            }
        });
    });

    let e = run_native(
        "Bread Launcer",
        opt,
        Box::new(move |cc| {
            let ctx = &cc.egui_ctx;
            install_image_loaders(ctx);
            let mut textures = vec![];

            for icon in ICONS {
                let uri = icon.0;
                let bytes = icon.1;
                let img = image::ImageReader::new(std::io::Cursor::new(bytes))
                    .with_guessed_format()?
                    .decode()?;

                let size = [img.width() as _, img.height() as _];
                let buffer = img.to_rgba8();
                let pixels = buffer.as_flat_samples();
                let img = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                textures.push(ctx.load_texture(uri, img, TextureOptions::LINEAR));
            }

            Ok(Box::new(BreadLauncher::new(cc.egui_ctx.clone(), textures)?))
        }),
    );

    if let Err(e) = e {
        log::error!("Failed to start bread launcher: {e:?}");
    }

    tx.send(())?;
    rt_thread.join()?;

    Ok(())
}
