use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result;
use eframe::{NativeOptions, run_native};
use egui::{ColorImage, TextureOptions};
use egui_extras::install_image_loaders;
use image::ImageReader;

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
    let app_icon = Arc::new(egui::IconData {
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
                let img = ImageReader::new(Cursor::new(bytes))
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

    Ok(())
}
