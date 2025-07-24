use std::fs::{create_dir_all, read, write};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Result, bail};
use reqwest::blocking::Client;

use crate::utils::sha1;

/// I'm sorry to my future self
/// It was good to be sorry
pub fn download(
    cl: &Client,
    path: impl AsRef<Path>,
    filename: impl AsRef<str>,
    url: impl AsRef<str>,
    attempts: u64,
) -> Result<()> {
    let pathc = path.as_ref().join(filename.as_ref());
    if pathc.is_file() {
        log::debug!(
            "{} already exists, no need to redownload...",
            filename.as_ref()
        );
        return Ok(());
    }

    create_dir_all(path.as_ref())?;
    log::info!("Requesting for {} from {}", filename.as_ref(), url.as_ref());
    let res = cl.get(url.as_ref()).send();
    if res.is_err() {
        if attempts > 4 {
            log::error!(
                "Max download attempts for file {} has been reached, I'm giving up",
                filename.as_ref()
            );

            bail!(res.unwrap_err());
        } else {
            log::warn!(
                "Download of {} is unsuccesful, sleeping for {} seconds",
                filename.as_ref(),
                10 * (attempts + 1)
            );
            sleep(Duration::from_secs(10 * attempts));
            log::warn!(
                "Attempting download of {} again (count: {})",
                filename.as_ref(),
                attempts + 1
            );

            return download(cl, path, filename, url, attempts + 1);
        }
    }

    let body = res.unwrap().bytes();
    if body.is_err() {
        if attempts > 4 {
            log::error!(
                "Max download attempts for file {} has been reached, I'm giving up",
                filename.as_ref()
            );

            bail!(body.unwrap_err());
        } else {
            log::warn!(
                "Download of {} is unsuccesful, sleeping for {} seconds",
                filename.as_ref(),
                10 * (attempts + 1)
            );
            sleep(Duration::from_secs(10 * attempts));
            log::warn!(
                "Attempting download of {} again (count: {})",
                filename.as_ref(),
                attempts + 1
            );

            return download(cl, path, filename, url, attempts + 1);
        }
    }

    let body = body.unwrap();
    write(pathc, &body)?;

    Ok(())
}

pub fn download_with_sha(
    cl: &Client,
    path: impl AsRef<Path>,
    filename: impl AsRef<str>,
    url: impl AsRef<str>,
    expected: impl AsRef<str>,
    attempts: u64,
) -> Result<()> {
    let pathc = path.as_ref().join(filename.as_ref());
    let path = path.as_ref().to_path_buf();
    if pathc.is_file() && attempts == 1 {
        log::debug!("{} already exists, gotta check the SHA1", filename.as_ref());
        let v = read(&pathc)?;
        let e = sha1::compare_sha1(expected.as_ref(), v.as_slice());
        if e.is_err() {
            if attempts > 4 {
                log::error!(
                    "Max download attempts for file {} has been reached, I'm giving up",
                    filename.as_ref()
                );

                bail!(e.unwrap_err());
            }

            log::error!(
                "SHA1 for file {} did not match, [attempts: {}]",
                filename.as_ref(),
                attempts + 1
            );
            log::error!("{e:?}");

            return download_with_sha(cl, path, filename, url, expected, attempts + 1);
        }

        log::debug!("{} passed the SHA1 test", filename.as_ref());
        return Ok(());
    }

    create_dir_all(&path)?;
    log::info!("Requesting for {} from {}", filename.as_ref(), url.as_ref());
    let res = cl.get(url.as_ref()).send();
    if res.is_err() {
        if attempts > 4 {
            log::error!(
                "Max download attempts for file {} has been reached, I'm giving up",
                filename.as_ref()
            );

            bail!(res.unwrap_err());
        } else {
            log::warn!(
                "Download of {} is unsuccesful, sleeping for {} seconds",
                filename.as_ref(),
                10 * attempts
            );
            sleep(Duration::from_secs(10 * attempts));
            log::warn!(
                "Attempting download of {} again (count: {})",
                filename.as_ref(),
                attempts + 1
            );

            return download_with_sha(cl, path, filename, url, expected, attempts + 1);
        }
    }

    let body = res.unwrap().bytes();
    if body.is_err() {
        if attempts > 4 {
            log::error!(
                "Max download attempts for file {} has been reached, I'm giving up",
                filename.as_ref()
            );

            bail!(body.unwrap_err());
        } else {
            log::warn!(
                "Download of {} is unsuccesful, sleeping for {} seconds",
                filename.as_ref(),
                10 * attempts
            );
            sleep(Duration::from_secs(10 * attempts));
            log::warn!(
                "Attempting download of {} again (count: {})",
                filename.as_ref(),
                attempts + 1
            );

            return download_with_sha(cl, path, filename, url, expected, attempts + 1);
        }
    }

    let body = body.unwrap();
    write(pathc, &body)?;
    let e = sha1::compare_sha1(expected.as_ref(), &body);
    if e.is_err() {
        if attempts > 4 {
            log::error!(
                "Max download attempts for file {} has been reached, I'm giving up",
                filename.as_ref()
            );

            bail!(e.unwrap_err());
        }

        log::error!(
            "SHA1 for file {} did not match, [attempts: {}]",
            filename.as_ref(),
            attempts + 1
        );
        log::error!("{e:?}");

        return download_with_sha(cl, path, filename, url, expected, attempts + 1);
    }

    log::debug!("{} passed the SHA1 test", filename.as_ref());
    Ok(())
}
