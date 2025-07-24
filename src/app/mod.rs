use std::any::Any;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpmc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use eframe::{App, Frame, NativeOptions, run_native};
use egui::Context;
use egui_extras::install_image_loaders;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use rand::{Rng, rng};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};
use uuid::Builder as UUBuilder;
use uuid::Version;

mod accounts;
mod add_instance;
mod settings;

use crate::account::Account;
use crate::app::accounts::AccountWin;
use crate::app::add_instance::AddInstance;
use crate::app::settings::SettingsWin;
use crate::assets::ICONS;
use crate::init::init_reqwest;
use crate::init::{get_appdir, init_logs};
use crate::instance::{Instance, InstanceLoader, Instances, UNGROUPED_NAME};
use crate::minecraft::MVOrganized;
use crate::settings::Settings;
use crate::utils::ShowWindow;
use crate::utils::message::Message;
use crate::widgets::selectable_image_label;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BreadLauncher {
    msg: Message,
    luuid: String,
    versions_last_update: u64,

    #[serde(default)]
    account: Arc<Account>,
    #[serde(default)]
    accounts: Arc<Mutex<Vec<Account>>>,
    #[serde(skip)]
    account_win: Arc<Mutex<AccountWin>>,
    #[serde(skip)]
    account_win_show: Arc<AtomicBool>,

    #[serde(skip)]
    instance: Arc<Instance>,
    #[serde(skip)]
    instance_selected: bool,
    instances: Arc<Mutex<Instances>>,
    #[serde(skip)]
    mvo: Arc<MVOrganized>,
    #[serde(skip, default = "BreadLauncher::aiw_default")]
    add_instance_win: Arc<Mutex<AddInstance>>,
    #[serde(skip)]
    add_instance_win_show: Arc<AtomicBool>,

    settings: Arc<Mutex<Settings>>,
    #[serde(skip)]
    settings_win: Arc<Mutex<SettingsWin>>,
    #[serde(skip)]
    settings_win_show: Arc<AtomicBool>,

    #[serde(skip)]
    context: Context,
    #[serde(skip)]
    client: Client,

    #[serde(skip, default = "BreadLauncher::channel_tx")]
    tx: Sender<Message>,
    #[serde(skip, default = "BreadLauncher::channel_rx")]
    rx: Receiver<Message>,

    #[serde(skip)]
    prog_step: Arc<AtomicUsize>,
    #[serde(skip)]
    prog_total: Arc<AtomicUsize>,
}

impl BreadLauncher {
    pub fn new(context: Context) -> Result<Self> {
        let client = init_reqwest()?;
        let appdir = get_appdir();
        let save = appdir.join("save.blauncher");
        let mut b = if !save.exists() {
            Self::new_clean(client.clone(), context)?
        } else {
            Self::load_launcher(appdir, context)?
        };

        let instances = b.instances.clone();
        let mut instances_lock = instances.lock().unwrap();
        instances_lock.cl = client.clone();

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let last = Duration::from_secs(b.versions_last_update);
        let ten_days = Duration::from_days(10);
        if ten_days <= (now - last) {
            instances_lock.parse_versions()?;
        } else {
            instances_lock.renew_version()?;
            b.versions_last_update = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        }

        b.mvo = b.mvo.renew(&client)?.into();

        Ok(b)
    }

    fn aiw_default() -> Arc<Mutex<AddInstance>> {
        Arc::new(Mutex::new(AddInstance::default()))
    }

    fn channel_tx() -> Sender<Message> {
        let (tx, _) = channel::<Message>();
        tx
    }

    fn channel_rx() -> Receiver<Message> {
        let (_, rx) = channel::<Message>();
        rx
    }

    fn save_launcher(&self) -> Result<()> {
        let f = File::create(get_appdir().join("save.blauncher"))?;
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
        let _ = File::open(appdir.as_ref().join("save.blauncher"))?.read_to_end(&mut compressed)?;
        let mut gz = GzDecoder::new(compressed.as_slice());
        let _ = gz.read_to_end(&mut decompressed)?;
        let mut de = Deserializer::from_slice(decompressed.as_slice());
        let mut b = Self::deserialize(&mut de)?;
        let (tx, rx) = channel::<Message>();
        b.context = ctx;
        b.tx = tx;
        b.rx = rx;
        b.prog_total.fetch_add(1, Ordering::Relaxed);

        Ok(b)
    }

    fn new_clean(client: Client, ctx: Context) -> Result<Self> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let mut rand = [0u8; 10];
        rng().fill(&mut rand);

        let (tx, rx) = channel::<Message>();
        let instances = Instances::new(client.clone())?;
        let uuid = UUBuilder::from_unix_timestamp_millis(time, &rand)
            .with_version(Version::SortRand)
            .into_uuid()
            .hyphenated()
            .to_string();

        let b = Self {
            msg: Message::default(),
            luuid: uuid,
            versions_last_update: 0,

            account: Account::default().into(),
            accounts: Mutex::new(vec![]).into(),
            account_win: Mutex::new(AccountWin::default()).into(),
            account_win_show: AtomicBool::new(false).into(),

            instance: Instance::default().into(),
            instance_selected: false,
            instances: Mutex::new(instances).into(),
            mvo: MVOrganized::default().renew(&client)?.into(),
            add_instance_win: Mutex::new(AddInstance::default()).into(),
            add_instance_win_show: AtomicBool::new(false).into(),

            settings: Mutex::new(Settings::default()).into(),
            settings_win: Mutex::new(SettingsWin {}).into(),
            settings_win_show: AtomicBool::new(false).into(),

            client,
            context: ctx,

            tx,
            rx,
            prog_step: AtomicUsize::new(0).into(),
            prog_total: AtomicUsize::new(1).into(),
        };

        Ok(b)
    }

    fn show_window<T: ShowWindow + Send + Sync + 'static>(
        &self,
        ctx: &Context,
        id: impl AsRef<str>,
        win: Arc<Mutex<T>>,
        show_win: Arc<AtomicBool>,
        data1: Arc<dyn Any + Send + Sync + 'static>,
        data2: Arc<dyn Any + Send + Sync + 'static>,
    ) {
        if !show_win.load(Ordering::Relaxed) {
            return;
        }

        let mctx = ctx.clone();
        let cl = self.client.clone();
        ctx.show_viewport_deferred(
            egui::ViewportId::from_hash_of(id.as_ref()),
            egui::ViewportBuilder::default().with_title(id.as_ref()),
            move |ctx, _cls| {
                win.lock().unwrap().show(
                    mctx.clone(),
                    ctx,
                    show_win.clone(),
                    data1.clone(),
                    data2.clone(),
                    cl.clone(),
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
        if let Ok(msg) = self.rx.try_recv() {
            self.msg = msg;
        }

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
                    egui::Label::new(format!("Minecraft Version: {}", self.instance.mc_ver)).wrap(),
                );

                if self.instance.loader != InstanceLoader::Vanilla {
                    ui.add(
                        egui::Label::new(format!(
                            "{:?} Version: {}",
                            self.instance.loader, self.instance.full_ver
                        ))
                        .wrap(),
                    );
                }
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
                    if ui.button("Start Offline").clicked() {
                        let instance = self.instance.clone();
                        let tx = self.tx.clone();
                        let ram = self.settings.lock().unwrap().jvm_ram;
                        let acc = self.account.clone();
                        spawn(move || {
                            log::info!("Run offline");
                            let _ = tx.send(Message::Message("Now launching instance".to_string()));
                            if let Err(e) = instance.run_offline(ram, acc.clone()) {
                                log::error!("{e:?}");
                                let _ = tx.send(Message::Errored(e.to_string()));
                            };
                        });
                    }

                    if ui.button("Start").clicked() {
                        let instance = self.instance.clone();
                        let cl = self.client.clone();
                        let step = self.prog_step.clone();
                        let total_steps = self.prog_total.clone();
                        let tx = self.tx.clone();
                        let ram = self.settings.lock().unwrap().jvm_ram;
                        let acc = self.account.clone();
                        spawn(move || {
                            log::info!("Run Online");
                            let e =
                                instance.run(cl, step, total_steps, tx.clone(), ram, acc.clone());
                            if let Err(e) = e {
                                log::error!("{e:?}");
                                let _ = tx.send(Message::Errored(e.to_string()));
                            };
                        });
                    }
                },
            );
        });

        egui::TopBottomPanel::bottom("Bread Launcher - Bottom Panel (Main)").show(ctx, |ui| {
            if self.msg == Message::default() {
                ui.label(format!(
                    "bread-launcher-{}: Nothing much happening right now...",
                    env!("CARGO_PKG_VERSION")
                ));
            } else {
                let step = self.prog_step.load(Ordering::Relaxed);
                let total = self.prog_total.load(Ordering::Relaxed);
                let prog = step as f32 / total as f32;
                let prog = egui::ProgressBar::new(prog)
                    .text(format!("{step:>4} / {total:>4}  -  {:3.2}%", prog * 100.0));
                match &self.msg {
                    Message::Message(msg) => {
                        ui.label(format!(
                            "bread-launcher-{}: {msg}",
                            env!("CARGO_PKG_VERSION"),
                        ));
                    }
                    Message::Downloading(msg) => {
                        ui.label(format!(
                            "bread-launcher-{}: {msg}",
                            env!("CARGO_PKG_VERSION"),
                        ));
                        ui.add(prog);
                    }
                    Message::Errored(msg) => {
                        ui.label(format!(
                            "bread-launcher-{}: {msg}",
                            env!("CARGO_PKG_VERSION"),
                        ));
                        ui.add(prog.fill(ui.style().visuals.error_fg_color));
                    }
                };
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Instances");
            });

            let instances = self.instances.lock().unwrap();
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

            egui::ScrollArea::vertical().show(ui, |ui|{
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
        });

        self.show_window(
            ctx,
            "Bread Launcher - Add Instance",
            self.add_instance_win.clone(),
            self.add_instance_win_show.clone(),
            self.instances.clone(),
            self.mvo.clone(),
        );

        self.show_window(
            ctx,
            "Bread Launcher - Settings",
            self.settings_win.clone(),
            self.settings_win_show.clone(),
            self.settings.clone(),
            Arc::new(0),
        );

        self.show_window(
            ctx,
            "Bread Launcher - Accounts",
            self.account_win.clone(),
            self.account_win_show.clone(),
            self.accounts.clone(),
            Arc::new(0),
        );

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.instance = Arc::new(Instance::default());
            self.instance_selected = false;
        }

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

pub fn run() -> Result<()> {
    init_logs()?;
    let opt = NativeOptions {
        persist_window: true,
        persistence_path: Some(get_appdir().join("save.ron")),
        vsync: true,
        ..Default::default()
    };

    let e = run_native(
        "Bread Launcer",
        opt,
        Box::new(move |cc| {
            let app = BreadLauncher::new(cc.egui_ctx.clone())?;
            install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );

    if let Err(e) = e {
        log::error!("Failed to start bread launcher: {e}");
    }

    Ok(())
}
