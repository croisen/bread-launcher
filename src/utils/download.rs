use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::result::Result as STRes;
use std::sync::Arc;

use anyhow::{Error, Result};
use reqwest::Client;
use tokio::fs::create_dir_all as tk_create_dir_all;
use tokio::fs::File as TkFile;
use tokio::fs::OpenOptions as TkOpenOptions;
use tokio::io::AsyncReadExt as TkAsyncReadExt;
use tokio::io::AsyncWriteExt as TkAsyncWriteExt;

use crate::utils::sha1;

/// I'm sorry to my future self
pub async fn download(
    cl: &Client,
    path: impl AsRef<Path>,
    filename: &str,
    url: &Arc<str>,
) -> Result<()> {
    let pathc = path.as_ref().join(filename);
    if pathc.is_file() {
        log::info!("{filename:#?} already exists, no need to redownload...");
        return Ok(());
    }

    tk_create_dir_all(path.as_ref()).await?;
    log::info!("Requesting for {filename} from {url}");
    let res = cl.get(url.as_ref()).send().await?;
    let body = res.bytes().await?;
    let mut of = TkOpenOptions::new()
        .write(true)
        .create(true)
        .open(pathc)
        .await?;

    of.write_all(&body).await?;
    of.flush().await?;
    Ok(())
}

pub fn download_with_sha<'a>(
    cl: &'a Client,
    path: impl AsRef<Path>,
    filename: &'a str,
    url: &'a Arc<str>,
    expected: &'a Arc<str>,
    use_regular: bool,
    attempts: usize,
) -> Pin<Box<dyn Future<Output = STRes<(), Error>> + 'a>> {
    let pathc = path.as_ref().join(filename);
    let path = path.as_ref().to_path_buf();
    let file = filename.to_string();
    let urlc = url.clone();
    let ex = expected.clone();

    Box::pin(async move {
        if pathc.is_file() && attempts == 1 {
            log::info!("{file} already exists, gotta check the SHA1");
            let mut f = TkFile::open(&pathc).await?;
            let mut v = vec![];
            f.read_to_end(&mut v).await?;
            if let Err(e) = sha1::compare_sha1(expected.as_ref(), v.as_slice(), true) {
                if attempts > 4 {
                    log::error!(
                        "Max download attempts for file {filename} has been reached, I'm giving up"
                    );

                    return Err(e);
                } else {
                    log::error!(
                        "SHA1 for file {filename} did not match, attempting again count: {}",
                        attempts + 1
                    );
                    log::error!("{e:?}");

                    return download_with_sha(
                        cl,
                        path,
                        filename,
                        url,
                        expected,
                        use_regular,
                        attempts + 1,
                    )
                    .await;
                }
            } else {
                log::info!("{filename} passed the SHA1 test");
                return Ok(());
            }
        }

        tk_create_dir_all(&path).await?;
        log::info!("Requesting for {file} from {urlc}");
        let res = cl.get(urlc.as_ref()).send().await?;
        let body = res.bytes().await?;
        let mut of = TkOpenOptions::new()
            .write(true)
            .create(true)
            .open(pathc)
            .await?;

        of.write_all(&body).await?;
        of.flush().await?;
        if let Err(e) = sha1::compare_sha1(expected.as_ref(), &body, true) {
            if attempts > 4 {
                log::error!(
                    "Max download attempts for file {filename} has been reached, I'm giving up"
                );

                return Err(e);
            } else {
                log::error!(
                    "SHA1 for file {filename} did not match, attempting again count: {}",
                    attempts + 1
                );
                log::error!("{e:?}");

                return download_with_sha(
                    cl,
                    path,
                    filename,
                    url,
                    expected,
                    use_regular,
                    attempts + 1,
                )
                .await;
            }
        } else {
            log::info!("{filename} passed the SHA1 test");
            return Ok(());
        }
    })
}
