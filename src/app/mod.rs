use std::any::Any;
use std::fs::File as STDFile;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel as std_channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
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
use crate::app::settings::SettingsWin;
use crate::assets::ICONS;
use crate::instance::{Instance, Instances, UNGROUPED_NAME};
use crate::logs::init_logs_and_appdir;
use crate::settings::Settings;
use crate::utils::ShowWindow;
use crate::utils::message::Message;
use crate::widgets::selectable_image_label;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BreadLauncher {
    msg: Message,
    luuid: String,
    collection: Instances,
    versions_last_update: u64,

    #[serde(default)]
    account: Account,
    #[serde(default, with = "crate::utils::serde_async_mutex")]
    accounts: Arc<TKMutex<Vec<Account>>>,
    #[serde(skip)]
    account_win: Arc<Mutex<AccountWin>>,
    #[serde(skip)]
    account_win_show: Arc<AtomicBool>,

    #[serde(skip)]
    instance: Arc<Instance>,
    #[serde(skip)]
    instance_selected: bool,
    #[serde(default, with = "crate::utils::serde_async_mutex")]
    instances: Arc<TKMutex<Instances>>,
    #[serde(skip, default = "BreadLauncher::aiw_default")]
    add_instance_win: Arc<Mutex<AddInstance>>,
    #[serde(skip)]
    add_instance_win_show: Arc<AtomicBool>,

    #[serde(default, with = "crate::utils::serde_async_mutex")]
    settings: Arc<TKMutex<Settings>>,
    #[serde(skip)]
    settings_win: Arc<Mutex<SettingsWin>>,
    #[serde(skip)]
    settings_win_show: Arc<AtomicBool>,

    #[serde(skip)]
    appdir: PathBuf,
    #[serde(skip)]
    context: Context,
    #[serde(skip)]
    client: Client,
    #[serde(skip, default = "Handle::current")]
    handle: Handle,
}

impl BreadLauncher {
    pub fn new(appdir: impl AsRef<Path>, context: Context) -> Result<Self> {
        let client = init::init_reqwest()?;
        let save = appdir.as_ref().join("save.blauncher");
        let b = if !save.exists() {
            Self::new_clean(client.clone(), appdir.as_ref(), context)?
        } else {
            Self::load_launcher(appdir.as_ref(), context)?
        };

        let handle = b.handle.clone();
        let instances = b.instances.clone();
        let mut instances_lock = handle.block_on(instances.lock());
        instances_lock.cl = client.clone();

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let last = Duration::from_secs(b.versions_last_update);
        let ten_days = Duration::from_days(10);
        if ten_days <= (now - last) {
            handle.block_on(instances_lock.parse_versions(appdir.as_ref()))?;
        } else {
            handle.block_on(instances_lock.renew_version(appdir.as_ref()))?;
        }

        Ok(b)
    }

    fn aiw_default() -> Arc<Mutex<AddInstance>> {
        Arc::new(Mutex::new(AddInstance::default()))
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

    fn load_launcher(appdir: impl AsRef<Path>, ctx: Context) -> Result<Self> {
        let mut compressed = vec![];
        let mut decompressed = vec![];
        let _ =
            STDFile::open(appdir.as_ref().join("save.blauncher"))?.read_to_end(&mut compressed)?;
        let mut gz = GzDecoder::new(compressed.as_slice());
        let _ = gz.read_to_end(&mut decompressed)?;
        let mut de = Deserializer::from_slice(decompressed.as_slice());
        let mut b = Self::deserialize(&mut de)?;
        b.appdir = appdir.as_ref().to_path_buf();
        b.context = ctx;

        Ok(b)
    }

    fn new_clean(client: Client, appdir: impl AsRef<Path>, ctx: Context) -> Result<Self> {
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
            instance_selected: false,
            instances: Arc::new(TKMutex::new(instances)),
            add_instance_win: Arc::new(Mutex::new(AddInstance::default())),
            add_instance_win_show: Arc::new(AtomicBool::new(false)),

            settings: Arc::new(TKMutex::new(Settings::default())),
            settings_win: Arc::new(Mutex::new(SettingsWin {})),
            settings_win_show: Arc::new(AtomicBool::new(false)),

            appdir: appdir.as_ref().to_path_buf(),
            client,
            context: ctx,
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
        let handle = self.handle.clone();
        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of(id.as_ref()),
            egui::ViewportBuilder::default().with_title(id.as_ref()),
            move |ctx, _cls| {
                win.lock().unwrap().show(
                    mctx.clone(),
                    ctx,
                    show_win.clone(),
                    data.clone(),
                    handle.clone(),
                );

                if ctx.input(|i| i.viewport().close_requested()) {
                    show_win.store(false, Ordering::Relaxed);
                }
            },
        );
    }
}

impl App for BreadLauncher {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.context.forget_all_images();
        log::info!("Saving launcher state");
        self.msg = Message::default();

        if let Err(e) = self.save_launcher() {
            log::error!("Failed to save launcher state {e}");
        }
    }

    fn update(&mut self, ctx: &Context, _fr: &mut Frame) {
        egui::TopBottomPanel::top("Bread Launcher - Top Panel (Main)").show(ctx, |ui| {
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

                            if ui.button("Settings").clicked() {
                                self.settings_win_show.store(true, Ordering::Relaxed);
                            }

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

        egui::SidePanel::right("Bread Launcher - Side Panel (Main)").show(ctx, |ui| {
            if !self.instance_selected {
                ui.disable();
            } else {
                ui.add(
                    egui::Label::new(format!("Name:    {}", self.instance.name))
                        .wrap_mode(egui::TextWrapMode::Wrap),
                );

                ui.add(
                    egui::Label::new(format!("Version: {}", self.instance.version))
                        .wrap_mode(egui::TextWrapMode::Wrap),
                );
            }

            ui.separator();

            ui.vertical_centered_justified(|ui| {
                if ui.button("Add Mods").clicked() {}

                if ui.button("Logs").clicked() {}

                if ui.button("Rename").clicked() {}

                if ui.button("Delete").clicked() {}
            });

            ui.with_layout(
                egui::Layout::bottom_up(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    if ui.button("Start Offline").clicked() {}

                    if ui.button("Start").clicked() {}
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Instances");
            });

            let instances = self.handle.block_on(self.instances.lock());
            let instances_lock = instances.get_instances();
            if instances_lock.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label("No instances found, use the button on the top left corner or the one over here to add one");
                    if ui.button("Add Instance").clicked() {
                        self.add_instance_win_show.store(true, Ordering::Relaxed);
                    }
                });

                return;
            }

            let mut last = None;
            for (group, instances) in instances_lock {
                if group == UNGROUPED_NAME {
                    last = Some(instances);
                    continue;
                }

                ui.heading(group);
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    for (name, instance) in instances {
                        if selectable_image_label(ui, &ICONS[0], name, &mut self.instance, instance.clone()).clicked() {
                            self.instance_selected = true;
                        }
                    }
                });
            }

            if let Some(instances) = last {
                ui.heading("Unnamed Group");
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    for (name, instance) in instances {
                        if selectable_image_label(ui, &ICONS[0], name, &mut self.instance, instance.clone()).clicked() {
                            self.instance_selected = true;
                        }
                    }
                });
            }
        });

        self.show_window(
            ctx,
            "Bread Launcher - Add Instance",
            self.add_instance_win.clone(),
            self.add_instance_win_show.clone(),
            self.instances.clone(),
        );

        self.show_window(
            ctx,
            "Bread Launcher - Settings",
            self.settings_win.clone(),
            self.settings_win_show.clone(),
            self.settings.clone(),
        );

        self.show_window(
            ctx,
            "Bread Launcher - Accounts",
            self.account_win.clone(),
            self.account_win_show.clone(),
            self.accounts.clone(),
        );

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.instance = Arc::new(Instance::default());
            self.instance_selected = false;
        }

        ctx.request_repaint_after(Duration::from_millis(50));
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
    let opt = NativeOptions {
        persist_window: true,
        persistence_path: Some(appdir.join("save.ron")),
        vsync: true,
        ..Default::default()
    };

    let e = run_native(
        "Bread Launcer",
        opt,
        Box::new(move |cc| {
            let app = BreadLauncher::new(&appdir, cc.egui_ctx.clone())?;
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
