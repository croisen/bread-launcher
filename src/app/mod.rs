use std::fs::File as STDFile;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel as std_channel;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use eframe::{App, Frame, NativeOptions, run_native};
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};
use egui_extras::install_image_loaders;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use rand::{Rng, rng};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};
use tokio::runtime::Builder as TKBuilder;
use tokio::runtime::Handle;
use tokio::time::sleep as tk_sleep;
use uuid::Builder as UUBuilder;
use uuid::Version;

mod init;

use crate::instance::Instances;
use crate::logs::init_logs_and_appdir;
use crate::utils::message::Message;

pub fn run() -> Result<()> {
    let appdir = init_logs_and_appdir()?;
    let (tx, rx) = std_channel::<Handle>();
    let (stx, srx) = std_channel::<()>();
    let rtt = thread::spawn::<_, Result<()>>(move || {
        let rt = TKBuilder::new_multi_thread()
            .worker_threads(16)
            .enable_all()
            .build()?;

        tx.send(rt.handle().clone())?;
        rt.block_on(async move {
            loop {
                tk_sleep(Duration::from_secs(1)).await;
                if srx.try_recv().is_ok() {
                    log::warn!("Stopping async runtime...");
                    break;
                }
            }
        });

        Ok(())
    });

    let handle = rx.recv()?;
    let _g = handle.enter();
    let app = BreadLauncher::new(&appdir)?;
    let opt = NativeOptions {
        persist_window: true,
        persistence_path: Some(appdir.join("save.ron")),
        vsync: true,
        ..Default::default()
    };

    let e = run_native(
        "Bread Launcer",
        opt,
        Box::new(|cc| {
            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );

    stx.send(())?;
    let _ = rtt.join();
    if let Err(e) = e {
        log::error!("Failed to start bread launcher: {e}");
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BreadLauncher {
    msg: Message,
    luuid: String,
    collection: Instances,

    #[serde(skip)]
    appdir: PathBuf,
    #[serde(skip)]
    client: Client,
    #[serde(skip, default = "Handle::current")]
    handle: Handle,
}

impl BreadLauncher {
    pub fn new(appdir: impl AsRef<Path>) -> Result<Self> {
        let client = init::init_reqwest()?;
        let save = appdir.as_ref().join("save.blauncher");
        let b = if !save.exists() {
            Self::new_clean(client, appdir)?
        } else {
            Self::load_launcher(appdir)?
        };

        Ok(b)
    }

    fn save_launcher(&self) -> Result<()> {
        let f = STDFile::create(self.appdir.join("save.blauncher"))?;
        let mut decompressed = vec![];
        let mut se = Serializer::pretty(&mut decompressed);
        self.serialize(&mut se)?;
        let mut gz = GzEncoder::new(f, Compression::best());
        gz.write_all(decompressed.as_slice())?;
        let _ = gz.finish()?;

        Ok(())
    }

    fn load_launcher(appdir: impl AsRef<Path>) -> Result<Self> {
        let mut compressed = vec![];
        let mut decompressed = vec![];
        let _ =
            STDFile::open(appdir.as_ref().join("save.blauncher"))?.read_to_end(&mut compressed)?;
        let mut gz = GzDecoder::new(compressed.as_slice());
        let _ = gz.read_to_end(&mut decompressed)?;
        let mut de = Deserializer::from_slice(decompressed.as_slice());
        let b = Self::deserialize(&mut de)?;

        Ok(b)
    }

    fn new_clean(client: Client, appdir: impl AsRef<Path>) -> Result<Self> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let mut rand = [0u8; 10];
        rng().fill(&mut rand);

        let handle = Handle::current();
        let collection = handle.block_on(Instances::new(client.clone(), &appdir))?;
        let uuid = UUBuilder::from_unix_timestamp_millis(time, &rand)
            .with_version(Version::SortRand)
            .into_uuid()
            .hyphenated()
            .to_string();

        let b = Self {
            msg: Message::default(),
            luuid: uuid,
            collection,

            appdir: appdir.as_ref().to_path_buf(),
            client,
            handle,
        };

        Ok(b)
    }
}

impl App for BreadLauncher {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.msg = Message::default();

        if let Err(e) = self.save_launcher() {
            log::error!("Failed to save launcher state {e}");
        }
    }

    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, World!");
        });
    }
}
