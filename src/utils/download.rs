use std::fs::{create_dir_all, read, write};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Result, bail};
use reqwest::blocking::Client;

use crate::utils::sha1::compare_sha1;

pub fn download(
    cl: &Client,
    path: impl AsRef<Path>,
    filename: impl AsRef<Path>,
    url: impl AsRef<str>,
    attempts: u64,
) -> Result<()> {
    let file = path.as_ref().join(&filename);
    if file.is_file() {
        return Ok(());
    }

    let bytes = __download(cl, &url);
    if let Err(e) = bytes {
        log::error!("Error in downloading file {:?}: {e:?}", filename.as_ref());
        if attempts < 4 {
            log::warn!(
                "Attempting download of file again in {} seconds",
                attempts * 5
            );

            sleep(Duration::from_secs(attempts * 5));
            return download(cl, path, filename, url, attempts + 1);
        }

        log::error!("Giving up on the download of file {:?}", filename.as_ref());
        bail!(e);
    }

    let bytes = bytes.unwrap();
    if !path.as_ref().is_dir() {
        create_dir_all(&path)?;
    }

    write(file, bytes)?;
    Ok(())
}

pub fn download_with_sha1(
    cl: &Client,
    path: impl AsRef<Path>,
    filename: impl AsRef<Path>,
    url: impl AsRef<str>,
    sha1: impl AsRef<str>,
    attempts: u64,
) -> Result<()> {
    let file = path.as_ref().join(&filename);
    if file.is_file() && compare_sha1(&sha1, read(&file)?).is_ok() {
        return Ok(());
    }

    let bytes = __download(cl, &url);
    if let Err(e) = bytes {
        log::error!("Error in downloading file {:?}: {e:?}", filename.as_ref());
        if attempts < 4 {
            log::warn!(
                "Attempting download of file again in {} seconds",
                attempts * 5
            );

            sleep(Duration::from_secs(attempts * 5));
            return download_with_sha1(cl, path, filename, url, sha1, attempts + 1);
        }

        log::error!("Giving up on the download of file {:?}", filename.as_ref());
        bail!(e);
    }

    let bytes = bytes.unwrap();
    if let Err(e) = compare_sha1(&sha1, &bytes) {
        log::error!("Error in downloading file {:?}: {e:?}", filename.as_ref());
        if attempts < 4 {
            log::warn!(
                "Attempting download of file again in {} seconds",
                attempts * 5
            );

            sleep(Duration::from_secs(attempts * 5));
            return download_with_sha1(cl, path, filename, url, sha1, attempts + 1);
        }

        log::error!("Giving up on the download of file {:?}", filename.as_ref());
        bail!(e);
    }

    if !path.as_ref().is_dir() {
        create_dir_all(&path)?;
    }

    write(file, bytes)?;
    Ok(())
}

fn __download(cl: &Client, url: impl AsRef<str>) -> Result<Vec<u8>> {
    Ok(cl
        .get(url.as_ref())
        .send()?
        .error_for_status()?
        .bytes()?
        .to_vec())
}
