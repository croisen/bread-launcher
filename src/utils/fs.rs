use std::fs::copy as fcopy;
use std::fs::create_dir_all;
use std::fs::read_dir;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;

use anyhow::{anyhow, Result};
use tokio::fs::copy as tk_fcopy;
use tokio::fs::create_dir_all as tk_create_dir_all;
use tokio::fs::read_dir as tk_read_dir;

/// ```
/// I wanted it to have the behaviour of cp -rf but I dunno if this is correct
/// but here's the summary
///
/// if s is a file and d doesn't exist, d will be created as a file
/// and the contents of s will be copied to it
///
/// if s is a file and d already exists and is a file, it will be overwritten
/// with the contents of s
///
/// if s is a directory and d doesn't exist then s will be copied to become d
///
/// if s is a directory and d exists then it will be copied as d/s
///
/// Copying a directory into a directory where it's parent directory doesn't
/// exist is fair game it will just be copied per the 3rd condition
///
/// if s is a directory and d is a file this function fails
/// ```
pub fn scopy(s: impl AsRef<Path>, d: impl AsRef<Path>) -> Result<()> {
    if s.as_ref().is_dir() {
        let created_dir = if !d.as_ref().is_dir() { true } else { false };
        if d.as_ref().is_file() {
            return Err(anyhow!(
                "Copying directory {:?} to file {:?}???",
                s.as_ref(),
                d.as_ref()
            ));
        }

        let mut p = s.as_ref().to_path_buf();
        if created_dir {
            create_dir_all(d.as_ref())?;
        } else {
            let _ = p.pop();
        }

        let prefix = p.to_string_lossy().to_string();
        for entry in read_dir(s.as_ref())? {
            match entry {
                Ok(f) => {
                    let fp = f.path();
                    let p = fp.strip_prefix(&prefix)?;
                    let dest = d.as_ref().join(p);
                    if fp.is_dir() {
                        scopy(&fp, &dest)?;
                        continue;
                    }

                    create_dir_all(fp.parent().unwrap())?;
                    log::debug!("Copying {fp:?} to {dest:?}");
                    fcopy(&fp, &dest)?;
                }
                Err(e) => {
                    log::error!("{e}");
                }
            }
        }
    } else {
        log::debug!("Copying {:?} to {:?}", s.as_ref(), d.as_ref());
        fcopy(s.as_ref(), d.as_ref())?;
    }

    Ok(())
}

/// Check scopy for the conditions, but this is the async version of it relying
/// on tokio's fs module, except for reading directories
pub async fn acopy<'a>(
    s: impl AsRef<Path> + 'a,
    d: impl AsRef<Path> + 'a,
) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>> {
    Box::pin(async move {
        if s.as_ref().is_dir() {
            let created_dir = if !d.as_ref().is_dir() { true } else { false };
            if d.as_ref().is_file() {
                return Err(anyhow!(
                    "Copying directory {:?} to file {:?}???",
                    s.as_ref(),
                    d.as_ref()
                ));
            }

            let mut p = s.as_ref().to_path_buf();
            if created_dir {
                tk_create_dir_all(d.as_ref()).await?;
            } else {
                let _ = p.pop();
            }

            let prefix = p.to_string_lossy().to_string();
            for entry in read_dir(s.as_ref())? {
                match entry {
                    Ok(f) => {
                        let fp = f.path();
                        let p = fp.strip_prefix(&prefix)?;
                        let dest = d.as_ref().join(p);
                        if fp.is_dir() {
                            acopy(&fp, &dest).await.await?;
                            continue;
                        }

                        tk_create_dir_all(fp.parent().unwrap()).await?;
                        log::debug!("Copying {fp:?} to {dest:?}");
                        tk_fcopy(&fp, &dest).await?;
                    }
                    Err(e) => {
                        log::error!("{e}");
                    }
                }
            }
        } else {
            log::debug!("Copying {:?} to {:?}", s.as_ref(), d.as_ref());
            tk_fcopy(s.as_ref(), d.as_ref()).await?;
        }

        Ok(())
    })
}
