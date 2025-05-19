#![allow(dead_code, unused_variables)]
#![feature(pattern)]

use std::env::args;
use std::path::Path;
use std::thread;

use anyhow::Result;
use reqwest::Client;

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
        .worker_threads(8)
        .build();
    if let Err(e) = &r {
        log::error!("Yabe: {:?}", e);
        return;
    }

    let h = thread::spawn(move || {
        let a = args().collect::<Vec<String>>();
        let r = r.unwrap();
        if let Err(e) = r.block_on(start_async(&a[1], &approot)) {
            log::error!("Yabe: {:?}", e);
        }
    });

    let _ = h.join();
}

async fn start_async(rel_ver: &str, appdir: impl AsRef<Path>) -> Result<()> {
    let cl = Client::builder()
        .user_agent("I am a bot yes")
        .https_only(true)
        .use_rustls_tls()
        .pool_max_idle_per_host(0)
        .build()?;

    let mvo: minecraft::MVOrganized =
        minecraft::MinecraftVersionManifest::new(&cl.clone(), &appdir)
            .await?
            .into();

    if let Some(c) = mvo.release.get(rel_ver) {
        let p = c.download(&cl.clone(), &appdir).await?;
        let m = minecraft::Minecraft::new(&p)?;
        m.download(&cl.clone(), &p).await?;
    } else {
        log::error!("Release ver {rel_ver} doesn't exist on the official version manifest...");
    }

    Ok(())
}
