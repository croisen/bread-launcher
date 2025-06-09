#![allow(dead_code, unused_variables)]
#![feature(pattern)]

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use reqwest::Client;

mod app;
mod assets;
mod instance;
mod logs;
mod minecraft;
mod utils;

fn main() {
    let r = logs::init_logs_and_appdir();
    if let Err(e) = &r {
        eprintln!("Failed to init logs: {e}");
        return;
    }

    let appdir = r.unwrap();
    match tokio::runtime::Builder::new_multi_thread()
        .thread_name("bread-launcher-main")
        .enable_all()
        .worker_threads(8)
        .build()
    {
        Ok(runtime) => {
            let (tx, rx) = mpsc::channel::<()>();
            let handle = runtime.handle().clone();
            let h = thread::spawn(move || {
                runtime.block_on(start_runtime(rx));
            });

            if let Err(e) = eframe::run_native(
                "Bread Launcher",
                eframe::NativeOptions {
                    viewport: egui::ViewportBuilder::default(),
                    persistence_path: Some(appdir.join("egui.ron")),
                    ..Default::default()
                },
                Box::new(move |_cc| {
                    let cl = Client::builder()
                        .user_agent(format!("bread-launcher-v-{}", env!("CARGO_PKG_VERSION")))
                        .https_only(true)
                        .use_rustls_tls()
                        .pool_max_idle_per_host(0)
                        .build()?;

                    Ok(Box::new(app::BreadLauncher::new(cl, appdir, &handle)?))
                }),
            ) {
                log::error!("{e}");
            }

            let _ = tx.send(());
            let _ = h.join();
        }
        Err(e) => {
            log::error!("Yabe: {e:?}");
        }
    }
}

async fn start_runtime(rx: mpsc::Receiver<()>) {
    loop {
        if let Ok(_) = rx.try_recv() {
            log::warn!("Quit signal received, stopping async loop");
            break;
        }

        tokio::time::sleep(Duration::new(1, 0)).await;
    }
}

/* If I'm feeling bored I'mma just launch the game with this cli-only
async fn start_async(appdir: impl AsRef<Path>, iname: &str, ver: &str) -> Result<()> {
    let cl = Client::builder()
        .user_agent(format!("bread-launcher-v-{}", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .use_rustls_tls()
        .pool_max_idle_per_host(0)
        .build()?;

    let mut i = instance::Instances::new(cl.clone(), appdir.as_ref()).await?;
    let m = match i.get_instance(iname) {
        Ok(m) => m.clone(),
        Err(_) => {
            let m = i
                .new_instance(
                    appdir.as_ref(),
                    "release",
                    ver,
                    iname,
                    instance::InstanceLoader::Vanilla,
                )
                .await?;
            m
        }
    };

    m.run(
        "1024M".to_string(),
        "Croisen".to_string(),
        "0".to_string(),
        "{}".to_string(),
    )
    .await?;
    i.save(appdir.as_ref()).await?;

    Ok(())
}

 */
