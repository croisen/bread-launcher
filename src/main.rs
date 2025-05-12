use std::env;
use std::error::Error;
use std::path::PathBuf;

use reqwest::Client;
use serde::Deserialize;
use serde_json::Deserializer;

mod launcher;

use launcher::MinecraftJson;

fn main() {
    let r = tokio::runtime::Builder::new_multi_thread()
        .thread_name("bread-launcher-main")
        .enable_all()
        .build()
        .unwrap()
        .block_on(start_async(env::current_dir().unwrap()));
    if let Err(e) = r {
        eprintln!("Yabe: {e}");
    }
}

async fn start_async(appdir: PathBuf) -> Result<(), Box<dyn Error>> {
    let cl = Client::new();
    let mut de = Deserializer::from_slice(include_bytes!("client.json"));
    let mj = MinecraftJson::deserialize(&mut de)?;
    let _ = mj.download_libs(cl, appdir).await?;
    Ok(())
}
