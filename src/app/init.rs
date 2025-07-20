use anyhow::Result;
use rand::{Rng, rng};
use reqwest::Client;

pub fn init_reqwest() -> Result<Client> {
    let i = rng().random::<u32>() % 2;
    let u = [
        format!("bread-launcher-{}", env!("CARGO_PKG_VERSION")),
        "I AM A ROBOT".to_string(),
    ];

    let c = Client::builder()
        .user_agent(&u[i as usize])
        .pool_idle_timeout(None)
        .use_rustls_tls()
        .https_only(true)
        .build()?;

    Ok(c)
}
