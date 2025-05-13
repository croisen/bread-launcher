use std::error::Error;
use std::path::Path;

use reqwest::ClientBuilder;

mod assets;
mod launcher;
mod logs;
mod utils;

fn main() {
    let r = logs::init_logs_and_appdir();
    if let Err(e) = &r {
        eprintln!("Failed to init logs: {e}");
    }

    let approot = r.unwrap();
    let r = tokio::runtime::Builder::new_multi_thread()
        .thread_name("bread-launcher-main")
        .enable_all()
        .build()
        .unwrap()
        .block_on(start_async(&approot));
    if let Err(e) = r {
        log::error!("Yabe: {e}");
    }
}

async fn start_async(appdir: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let cl = ClientBuilder::new().user_agent("Hello There!").build()?;
    let mv = launcher::MinecraftVersions::new(assets::load_versions())?;
    if let Some(mc) = mv.release.get("1.21.5") {
        log::info!("Version 1.21.5 is available");
        let c = mc.download(cl.clone(), &appdir).await?;
        let mj = launcher::MinecraftJson::new(&c)?;
        mj.download_libs(cl.clone(), &c).await?;
    } else {
        log::error!("Could not get specific version 1.21.5");
        log::error!("Croisen was feeling lazy to update it yet");
    }

    Ok(())
}
