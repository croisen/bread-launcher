use std::fs::copy as fcopy;
use std::fs::create_dir_all;
use std::fs::read_dir;
use std::path::Path;

use anyhow::{Result, anyhow};

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
        let created_dir = !d.as_ref().is_dir();
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

        let prefix = p.to_str().unwrap();
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
