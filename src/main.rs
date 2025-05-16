#![allow(dead_code, unused_variables)]

use std::path::Path;
use std::thread;

use anyhow::Result;
use reqwest::ClientBuilder;

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
        log::error!("Yabe: {:?}", e);
        return;
    }

    let h = thread::spawn(move || {
        let r = r.unwrap();
        if let Err(e) = r.block_on(start_async(&approot)) {
            log::error!("Yabe: {:?}", e);
        }
    });

    let _ = h.join();
}

async fn start_async(appdir: impl AsRef<Path>) -> Result<()> {
    let cl = ClientBuilder::new()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36")
        .https_only(true)
        .use_rustls_tls()
        .build()?;

    let mvo: minecraft::MVOrganized =
        minecraft::MinecraftVersionManifest::new(&cl.clone(), &appdir)
            .await?
            .into();

    let c = mvo.release.get("1.14.2").unwrap();
    let p = c.download(&cl.clone(), &appdir).await?;
    let m = minecraft::Minecraft::new(&p)?;
    m.download(&cl.clone(), &p).await?;

    Ok(())
}
