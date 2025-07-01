#![allow(dead_code, unused_variables)]
#![feature(pattern)]

use anyhow::{Context, Result};
use reqwest::Client;

mod app;
mod assets;
mod instance;
mod logs;
mod minecraft;
mod utils;

fn main() -> Result<()> {
    let appdir = logs::init_logs_and_appdir()?;
    let cl = Client::builder()
        .user_agent(format!("bread-launcher-v-{}", env!("CARGO_PKG_VERSION")))
        .https_only(true)
        .use_rustls_tls()
        .pool_max_idle_per_host(0)
        .build()
        .context("Could not build reqwest client")?;

    Ok(())
}
