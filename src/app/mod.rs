use std::any::Any;
use std::fs::File as STDFile;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel as std_channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use eframe::{App, Frame, NativeOptions, run_native};
use egui::Context;
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
use tokio::sync::Mutex as TKMutex;
use tokio::time::sleep as tk_sleep;
use uuid::Builder as UUBuilder;
use uuid::Version;

mod accounts;
mod add_instance;
mod init;
mod settings;

use crate::account::Account;
use crate::app::accounts::AccountWin;
use crate::app::add_instance::AddInstance;
use crate::instance::{Instance, Instances};
use crate::logs::init_logs_and_appdir;
use crate::utils::ShowWindow;
use crate::utils::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BreadLauncher {
    msg: Message,
    luuid: String,
    collection: Instances,
    versions_last_update: u64,

    account: Account,
    #[serde(with = "crate::utils::serde_async_mutex")]
    accounts: Arc<TKMutex<Vec<Account>>>,
    #[serde(skip)]
    account_win: Arc<Mutex<AccountWin>>,
    #[serde(skip)]
    account_win_show: Arc<AtomicBool>,

    #[serde(skip)]
    instance: Arc<Instance>,
    #[serde(with = "crate::utils::serde_async_mutex")]
    instances: Arc<TKMutex<Instances>>,
    #[serde(skip)]
    add_instance_win: Arc<Mutex<AddInstance>>,
    #[serde(skip)]
    add_instance_win_show: Arc<AtomicBool>,

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
            Self::new_clean(client, appdir.as_ref())?
        } else {
            Self::load_launcher(appdir.as_ref())?
        };

        let handle = b.handle.clone();
        let instances = b.instances.clone();
        let mut instances_lock = handle.block_on(instances.lock());

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let last = Duration::from_secs(b.versions_last_update);
        let ten_days = Duration::from_days(10);
        if ten_days <= (now - last) {
            handle.block_on(instances_lock.renew_version(appdir.as_ref()))?;
        } else {
            handle.block_on(instances_lock.parse_versions(appdir.as_ref()))?;
        }

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

        let instances = handle.block_on(Instances::new(client.clone(), appdir.as_ref()))?;
        let b = Self {
            msg: Message::default(),
            luuid: uuid,
            collection,
            versions_last_update: 0,

            account: Account::default(),
            accounts: Arc::new(TKMutex::new(vec![])),
            account_win: Arc::new(Mutex::new(AccountWin::default())),
            account_win_show: Arc::new(AtomicBool::new(false)),

            instance: Arc::new(Instance::default()),
            instances: Arc::new(TKMutex::new(instances)),
            add_instance_win: Arc::new(Mutex::new(AddInstance::default())),
            add_instance_win_show: Arc::new(AtomicBool::new(false)),

            appdir: appdir.as_ref().to_path_buf(),
            client,
            handle,
        };

        Ok(b)
    }

    fn show_window<T: ShowWindow + Send + Sync + 'static>(
        &self,
        ctx: &Context,
        id: impl AsRef<str>,
        win: Arc<Mutex<T>>,
        show_win: Arc<AtomicBool>,
        data: Arc<dyn Any + Send + Sync + 'static>,
    ) {
        if !show_win.load(Ordering::Relaxed) {
            return;
        }

        let mctx = ctx.clone();
        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of(id.as_ref()),
            egui::ViewportBuilder::default(),
            move |ctx, _cls| {
                win.lock()
                    .unwrap()
                    .show(mctx.clone(), ctx, show_win.clone(), data.clone());

                if ctx.input(|i| i.viewport().close_requested()) {
                    show_win.store(false, Ordering::Relaxed);
                }
            },
        );
    }
}

impl App for BreadLauncher {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log::info!("Saving launcher state");
        self.msg = Message::default();

        if let Err(e) = self.save_launcher() {
            log::error!("Failed to save launcher state {e}");
        }
    }

    fn update(&mut self, ctx: &Context, _fr: &mut Frame) {
        egui::TopBottomPanel::top("Bread Launcher - Top Panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(
                    egui::Layout::default()
                        .with_cross_align(egui::Align::Min)
                        .with_main_align(egui::Align::Center),
                    |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Add Instance").clicked() {
                                self.add_instance_win_show.store(true, Ordering::Relaxed);
                            }

                            if ui.button("Settings").clicked() {}

                            if ui.button("About").clicked() {}
                        });
                    },
                );

                ui.with_layout(
                    egui::Layout::default()
                        .with_cross_align(egui::Align::Max)
                        .with_main_align(egui::Align::Center),
                    |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Account").clicked() {
                                self.account_win_show.store(true, Ordering::Relaxed);
                            }
                        });
                    },
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, World!");
        });
    }
}

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
