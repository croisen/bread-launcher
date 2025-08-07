#![allow(dead_code)]
#![cfg_attr(
    not(debug_assertions),
    cfg_attr(target_family = "windows", windows_subsystem = "windows")
)]

mod app;
mod assets;
mod minecraft;
mod utils;
mod widgets;

mod account;
mod init;
mod instance;
mod settings;

fn main() {
    #[cfg(debug_assertions)]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1")
    };

    if let Err(e) = run() {
        eprintln!("{e:#?}");
    }
}

fn run() -> Result<(), anyhow::Error> {
    init::init_logs()?;

    let img = image::ImageReader::new(std::io::Cursor::new(assets::ICONS[0].1))
        .with_guessed_format()?
        .decode()?;

    let size = [img.width() as _, img.height() as _];
    let buffer = img.to_rgba8();
    let pixels = buffer.as_flat_samples();
    let app_icon = std::sync::Arc::new(egui::IconData {
        rgba: pixels.to_vec().samples,
        width: size[0],
        height: size[1],
    });

    let icon = app_icon.clone();
    let opt = eframe::NativeOptions {
        persist_window: true,
        persistence_path: Some(init::get_appdir().join("save.ron")),
        vsync: true,
        window_builder: Some(Box::new(move |vb| vb.with_icon(icon))),
        viewport: egui::ViewportBuilder {
            icon: Some(app_icon),
            ..Default::default()
        },
        ..Default::default()
    };

    let e = eframe::run_native(
        "Bread Launcer",
        opt,
        Box::new(move |cc| {
            let ctx = &cc.egui_ctx;
            egui_extras::install_image_loaders(ctx);
            let mut textures = vec![];

            for icon in assets::ICONS {
                let uri = icon.0;
                let bytes = icon.1;
                let img = image::ImageReader::new(std::io::Cursor::new(bytes))
                    .with_guessed_format()?
                    .decode()?;

                let size = [img.width() as _, img.height() as _];
                let buffer = img.to_rgba8();
                let pixels = buffer.as_flat_samples();
                let img = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
                textures.push(ctx.load_texture(uri, img, egui::TextureOptions::LINEAR));
            }

            Ok(Box::new(app::BreadLauncher::new(
                cc.egui_ctx.clone(),
                textures,
            )?))
        }),
    );

    if let Err(e) = e {
        log::error!("Failed to start bread launcher: {e:?}");
    }

    Ok(())
}
