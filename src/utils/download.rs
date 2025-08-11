use std::{path::Path, pin::Pin};

use anyhow::{Context, Result, anyhow, bail};
use reqwest::Client;
use tokio::fs::{create_dir_all, read, write};

use crate::utils::sha1::compare_sha1;

pub async fn download<'a>(
    cl: Client,
    path: impl AsRef<Path> + 'a,
    filename: impl AsRef<str> + 'a,
    url: impl AsRef<str> + 'a,
    attempts: u64,
) -> Result<()> {
    Box::pin(async move {
        let fpath = path.as_ref().join(filename.as_ref());
        if fpath.exists() {
            return Ok(());
        }

        let bytes = __download(cl.clone(), &url).await;
        if bytes.is_err() {
            if attempts < 4 {
                let err = bytes.unwrap_err();
                log::error!("Download of {:?} errored with {err:?}", filename.as_ref());
                log::warn!(
                    "Attempting download of {:?} again after {} seconds",
                    filename.as_ref(),
                    attempts * 5
                );

                return download(cl, path, filename, url, attempts + 1).await;
            }

            log::error!(
                "Max download attempts for {:?} have been reached",
                filename.as_ref(),
            );

            bail!(bytes.unwrap_err());
        }

        create_dir_all(&path).await?;
        write(fpath, bytes.unwrap()).await?;

        Ok(())
    })
}

pub fn download_with_sha1<'a>(
    cl: Client,
    path: impl AsRef<Path> + 'a,
    filename: impl AsRef<str> + 'a,
    url: impl AsRef<str> + 'a,
    expected: impl AsRef<str> + 'a,
    attempts: u64,
) -> Pin<Box<impl Future<Output = Result<()>> + 'a>> {
    Box::pin(async move {
        let fpath = path.as_ref().join(filename.as_ref());
        if !__check_sha1(&path, &filename, &expected).await.is_err() {
            return Ok(());
        }

        let bytes = __download(cl.clone(), &url).await;
        if bytes.is_err() {
            if attempts < 4 {
                let err = bytes.unwrap_err();
                log::error!("Download of {:?} errored with {err:?}", filename.as_ref());
                log::warn!(
                    "Attempting download of {:?} again after {} seconds",
                    filename.as_ref(),
                    attempts * 5
                );

                return download_with_sha1(cl, path, filename, url, expected, attempts + 1).await;
            }

            log::error!(
                "Max download attempts for {:?} have been reached",
                filename.as_ref(),
            );

            bail!(bytes.unwrap_err());
        }

        let sha = __check_sha1(&path, &filename, &expected).await;
        if sha.is_err() {
            if attempts < 4 {
                let err = sha.unwrap_err();
                log::error!(
                    "Verification of {:?} errored with {err:?}",
                    filename.as_ref()
                );
                log::warn!(
                    "Attempting download of {:?} again after {} seconds",
                    filename.as_ref(),
                    attempts * 5
                );

                return download_with_sha1(cl, path, filename, url, expected, attempts + 1).await;
            }

            log::error!(
                "Max download attempts for {:?} have been reached",
                filename.as_ref(),
            );

            bail!(sha.unwrap_err());
        }

        create_dir_all(&path).await?;
        write(fpath, bytes.unwrap()).await?;

        Ok(())
    })
}

async fn __check_sha1(
    path: impl AsRef<Path>,
    filename: impl AsRef<str>,
    expected: impl AsRef<str>,
) -> Result<()> {
    let fpath = path.as_ref().join(filename.as_ref());
    if !fpath.exists() {
        bail!("{fpath:?} doesn't exist");
    }

    let contents = read(&fpath)
        .await
        .context(anyhow!("Was reading {fpath:?} to check it's SHA1 hash"))?;

    compare_sha1(&expected, &contents).context(anyhow!("It was the SHA1 of the file {fpath:?}"))?;

    Ok(())
}

// Return the downloaded bytes
async fn __download(cl: Client, url: impl AsRef<str>) -> Result<Box<[u8]>> {
    let bytes = cl
        .get(url.as_ref())
        .send()
        .await
        .context(anyhow!("Could not send request to {}", url.as_ref()))?
        .bytes()
        .await
        .context("Responde from url was weird")?;

    Ok(bytes.as_ref().into())
}
