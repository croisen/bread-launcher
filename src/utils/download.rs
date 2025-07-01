use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::time::Duration;

use anyhow::Result;
use reqwest::Client;
use tokio::fs::create_dir_all as tk_create_dir_all;
use tokio::fs::read as tk_read;
use tokio::fs::write as tk_write;

use crate::utils::sha1;

/// I'm sorry to my future self
pub fn download<'a>(
    cl: &'a Client,
    path: impl AsRef<Path> + Send + Sync + 'a,
    filename: impl AsRef<str> + Send + Sync + 'a,
    url: impl AsRef<str> + Send + Sync + 'a,
    _attempts: u64,
) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let pathc = path.as_ref().join(filename.as_ref());
        if pathc.is_file() {
            log::debug!(
                "{} already exists, no need to redownload...",
                filename.as_ref()
            );
            return Ok(());
        }

        tk_create_dir_all(path.as_ref()).await?;
        log::info!("Requesting for {} from {}", filename.as_ref(), url.as_ref());
        let res = cl.get(url.as_ref()).send().await?;
        let body = res.bytes().await?;
        tk_write(pathc, &body).await?;

        Ok(())
    })
}

pub fn download_with_sha<'a>(
    cl: &'a Client,
    path: impl AsRef<Path> + Send + Sync + 'a,
    filename: impl AsRef<str> + Send + Sync + 'a,
    url: impl AsRef<str> + Send + Sync + 'a,
    expected: impl AsRef<str> + Send + Sync + 'a,
    _use_regular: bool,
    _attempts: u64,
) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let pathc = path.as_ref().join(filename.as_ref());
        let path = path.as_ref().to_path_buf();
        if pathc.is_file() && _attempts == 1 {
            log::debug!("{} already exists, gotta check the SHA1", filename.as_ref());
            let v = tk_read(&pathc).await?;
            if let Err(e) = sha1::compare_sha1(expected.as_ref(), v.as_slice(), true) {
                if _attempts > 4 {
                    log::error!(
                        "Max download attempts for file {} has been reached, I'm giving up",
                        filename.as_ref()
                    );

                    return Err(e);
                } else {
                    log::error!(
                        "SHA1 for file {} did not match, [attempts: {}]",
                        filename.as_ref(),
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
                log::debug!("{} passed the SHA1 test", filename.as_ref());
                return Ok(());
            }
        }

        tk_create_dir_all(&path).await?;
        log::info!("Requesting for {} from {}", filename.as_ref(), url.as_ref());
        match cl.get(url.as_ref()).send().await {
            Err(e) => {
                if _attempts > 4 {
                    log::error!(
                        "Max download attempts for file {} has been reached, I'm giving up",
                        filename.as_ref()
                    );

                    return Err(e.into());
                }

                log::error!(
                    "Got error while requesting to {}, retrying [attempts: {}]",
                    filename.as_ref(),
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
                            "Max download attempts for file {} has been reached, I'm giving up",
                            filename.as_ref()
                        );

                        Err(e)
                    } else {
                        log::error!(
                            "SHA1 for file {} did not match, [attempts: {}]",
                            filename.as_ref(),
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
                    log::debug!("{} passed the SHA1 test", filename.as_ref());
                    Ok(())
                }
            }
        }
    })
}
