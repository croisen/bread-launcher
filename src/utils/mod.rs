use std::error::Error;
use std::path::Path;

pub mod download;
pub mod sha1;

pub async fn copy_dir_recursive(
    dest: impl AsRef<Path>,
    src: impl AsRef<Path>,
) -> Result<(), Box<dyn Error>> {
    Ok(())
}
