use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::result::Result as STRes;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Error, Result};
use reqwest::Client;
use tokio::fs::create_dir_all as tk_create_dir_all;
use tokio::fs::read as tk_read;
use tokio::fs::write as tk_write;

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
        log::debug!("{filename:#?} already exists, no need to redownload...");
        return Ok(());
    }

    tk_create_dir_all(path.as_ref()).await?;
    log::info!("Requesting for {filename} from {url}");
    let res = cl.get(url.as_ref()).send().await?;
    let body = res.bytes().await?;
    tk_write(pathc, &body).await?;
    Ok(())
}

/// I'm kinda proud of this one
/// Now it's a nightmare for the never nesters
pub fn download_with_sha<'a>(
    cl: &'a Client,
    path: impl AsRef<Path>,
    filename: &'a str,
    url: &'a Arc<str>,
    expected: &'a Arc<str>,
    _use_regular: bool,
    _attempts: u64,
) -> Pin<Box<dyn Future<Output = STRes<(), Error>> + Send + 'a>> {
    let pathc = path.as_ref().join(filename);
    let path = path.as_ref().to_path_buf();
    let urlc = url.clone();
    let ex = expected.clone();

    Box::pin(async move {
        if pathc.is_file() && _attempts == 1 {
            log::debug!("{filename} already exists, gotta check the SHA1");
            let v = tk_read(&pathc).await?;
            if let Err(e) = sha1::compare_sha1(expected.as_ref(), v.as_slice(), true) {
                if _attempts > 4 {
                    log::error!(
                        "Max download attempts for file {filename} has been reached, I'm giving up"
                    );

                    return Err(e);
                } else {
                    log::error!(
                        "SHA1 for file {filename} did not match, [attempts: {}]",
                        _attempts + 1
                    );
                    log::error!("{e:?}");

                    return download_with_sha(
                        cl,
                        path,
                        filename,
                        url,
                        expected,
                        _use_regular,
                        _attempts + 1,
                    )
                    .await;
                }
            } else {
                log::debug!("{filename} passed the SHA1 test");
                return Ok(());
            }
        }

        tk_create_dir_all(&path).await?;
        log::info!("Requesting for {filename} from {urlc}");
        match cl.get(urlc.as_ref()).send().await {
            Err(e) => {
                if _attempts > 4 {
                    log::error!(
                        "Max download attempts for file {filename} has been reached, I'm giving up"
                    );

                    return Err(e.into());
                }

                log::error!(
                    "Got error while requesting to {url}, retrying [attempts: {}]",
                    _attempts + 1
                );
                log::error!("{e:?}");
                log::error!(
                    "Sleeping for {} seconds before retrying download [this error may be a rate limit error]",
                    _attempts * 10
                );
                tokio::time::sleep(Duration::new(_attempts * 10, 0)).await;

                download_with_sha(
                    cl,
                    path,
                    filename,
                    url,
                    expected,
                    _use_regular,
                    _attempts + 1,
                )
                .await
            }

            Ok(res) => {
                let body = res.bytes().await?;
                tk_write(pathc, &body).await?;
                if let Err(e) = sha1::compare_sha1(expected.as_ref(), &body, true) {
                    if _attempts > 4 {
                        log::error!(
                            "Max download attempts for file {filename} has been reached, I'm giving up"
                        );

                        Err(e)
                    } else {
                        log::error!(
                            "SHA1 for file {filename} did not match, [attempts: {}]",
                            _attempts + 1
                        );
                        log::error!("{e:?}");

                        download_with_sha(
                            cl,
                            path,
                            filename,
                            url,
                            expected,
                            _use_regular,
                            _attempts + 1,
                        )
                        .await
                    }
                } else {
                    log::debug!("{filename} passed the SHA1 test");
                    Ok(())
                }
            }
        }
    })
}
