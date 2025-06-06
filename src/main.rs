#![allow(dead_code, unused_variables)]
#![feature(pattern)]

use std::env::args;
use std::path::Path;
use std::thread;

use anyhow::Result;
use reqwest::Client;

mod assets;
mod instance;
mod logs;
mod minecraft;
mod utils;

fn main() {
    let r = logs::init_logs_and_appdir();
    if let Err(e) = &r {
        eprintln!("Failed to init logs: {e}");
        return;
    }

    let approot = r.unwrap();
    let r = tokio::runtime::Builder::new_multi_thread()
        .thread_name("bread-launcher-main")
        .enable_all()
        .worker_threads(8)
        .build();
    if let Err(e) = &r {
        log::error!("Yabe: {e:?}");
        return;
    }

    let h = thread::spawn(move || {
        let a = args().collect::<Vec<String>>();
        let r = r.unwrap();
        if let Err(e) = r.block_on(start_async(approot, &a[1], &a[2])) {
            log::error!("Yabe: {e:?}");
        }
    });

    let _ = h.join();
}

async fn start_async(appdir: impl AsRef<Path>, iname: &str, ver: &str) -> Result<()> {
    let cl = Client::builder()
        .user_agent("I am a bot yes")
        .https_only(true)
        .use_rustls_tls()
        .pool_max_idle_per_host(0)
        .build()?;

    let mut i = instance::Instances::new(cl.clone(), appdir.as_ref()).await?;
    let m = match i.get_instance(iname) {
        Ok(m) => m.clone(),
        Err(_) => {
            let m = i
                .new_instance(
                    appdir.as_ref(),
                    "release",
                    ver,
                    iname,
                    instance::InstanceLoader::Vanilla,
                )
                .await?;
            m
        }
    };

    m.run("1024M".to_string(), "Croisen".to_string()).await?;
    i.save(appdir.as_ref()).await?;

    Ok(())
}
