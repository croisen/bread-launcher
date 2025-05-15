use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use tokio::fs::create_dir_all as tk_create_dir_all;
use tokio::fs::OpenOptions as TkOpenOptions;
use tokio::io::AsyncWriteExt as TkAsyncWriteExt;
use tokio::task::JoinHandle;

/// I'm sorry to my future self
pub async fn download(
    cl: &Client,
    path: impl AsRef<Path> + Debug,
    filename: &str,
    url: &Arc<str>,
) -> JoinHandle<Result<()>> {
    let clc = cl.clone();
    let mut pathc = path.as_ref().to_path_buf();
    let fc = filename.to_string();
    let urlc = url.clone();

    tokio::spawn(async move {
        pathc.push(&fc);
        if pathc.is_file() {
            log::info!("{fc:#?} already exists, no need to redownload...");
            return Ok(());
        }

        let _ = pathc.pop();
        tk_create_dir_all(&pathc).await?;
        pathc.push(&fc);
        log::info!("Requesting for {fc} from {urlc}");
        let res = clc.get(urlc.as_ref()).send().await?;
        let body = res.bytes().await?;
        let mut of = TkOpenOptions::new()
            .write(true)
            .create(true)
            .open(pathc)
            .await?;

        of.write_all(&body).await?;
        of.flush().await?;
        Ok(())
    })
}
