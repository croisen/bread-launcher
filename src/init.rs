use std::env::var_os;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use reqwest::blocking::Client;

pub fn init_appdir() -> Result<PathBuf> {
    #[cfg(target_family = "windows")]
    let appdir = {
        let mut p = PathBuf::from(
            var_os("APPDATA").ok_or(anyhow!("Variable %APPDATA% doesn't exist?? How??"))?,
        );

        p.push("Bread Launcher");
        p
    };

    #[cfg(target_family = "unix")]
    let appdir = {
        let mut p = PathBuf::from(
            var_os("HOME").ok_or(anyhow!("How does the $HOME variable doesn't exist??"))?,
        );

        p.push(".local");
        p.push("share");
        p.push("breadlauncher");
        p
    };

    Ok(appdir)
}

pub fn init_reqwest() -> Result<Client> {
    #[cfg(target_family = "windows")]
    let user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";
    #[cfg(target_family = "unix")]
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

    let c = Client::builder()
        .user_agent(user_agent)
        .pool_idle_timeout(None)
        .use_rustls_tls()
        .https_only(true)
        .build()?;

    Ok(c)
}
