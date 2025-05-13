use std::error::Error;
use std::fmt::Debug;
use std::path::Path;
use std::sync::Arc;

use crypto::digest::Digest;
use crypto::sha1::Sha1;
use reqwest::Client;
use tokio::fs::create_dir_all as tk_create_dir_all;
use tokio::fs::OpenOptions as TkOpenOptions;
use tokio::io::AsyncWriteExt as TkAsyncWriteExt;
use tokio::task::JoinHandle;

mod sha1mismatch;
pub use sha1mismatch::SHA1Mismatch;

/// I'm sorry to my future self
pub async fn download(
    cl: &Client,
    path: impl AsRef<Path> + Debug,
    filename: &str,
    url: &Arc<str>,
) -> JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
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
        let body = res.text().await?;

        let mut of = TkOpenOptions::new()
            .write(true)
            .create(true)
            .open(pathc)
            .await?;
        of.write_all(body.as_bytes()).await?;
        of.sync_all().await?;
        Ok(())
    })
}

pub fn regular_sha1(data: &[u8]) -> String {
    let mut sha1 = Sha1::new();
    sha1.input(data);
    sha1.result_str()
}

pub fn notchian_sha1(data: &[u8]) -> String {
    let mut sha1 = Sha1::new();
    sha1.input(data);
    let mut digest = [0u8; 20];
    sha1.result(&mut digest);
    let negative = digest[0] & 0x80 != 0;
    let mut hex = String::with_capacity(40 + negative as usize);
    if negative {
        hex.push('-');
        digest[0] &= 0b0111_1111;
        let mut carry = true;
        for b in digest.iter_mut().rev() {
            (*b, carry) = (!*b).overflowing_add(carry as u8);
        }
    }

    hex.extend(
        digest
            .into_iter()
            .flat_map(|x| [x >> 4, x & 0xf])
            .skip_while(|&x| x == 0)
            .map(|x| char::from_digit(x as u32, 16).expect("x is valid base16 tho?")),
    );

    hex
}
