use std::fs::File;
use std::io::{Read, Write};
use std::mem::swap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use eframe::{App, Frame};
use egui::Context;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use parking_lot::Mutex;
use rand::{Rng, rng};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};
use uuid::Builder as UUBuilder;

mod about;
mod accounts;
mod add_instance;
mod launch;
mod settings;

use crate::account::Account;
use crate::app::about::AboutWin;
use crate::app::accounts::AccountWin;
use crate::app::add_instance::AddInstance;
use crate::app::settings::SettingsWin;
use crate::init::{FULLNAME, UNGROUPED_NAME, get_appdir, init_reqwest};
use crate::instance::{Instance, InstanceLoader, Instances};
use crate::minecraft::MVOrganized;
use crate::settings::Settings;
use crate::utils::message::Message;
use crate::utils::{ShowWindow, WindowData};
use crate::widgets::SelectableImageLabel;

pub use crate::app::launch::launch;

#[derive(Serialize, Deserialize)]
pub struct BreadLauncher {
    msg: Message,
    luuid: String,
    versions_last_update: u64,

    #[serde(skip)]
    about_win: Arc<Mutex<AboutWin>>,
    #[serde(skip)]
    about_win_show: Arc<AtomicBool>,

    #[serde(default)]
    account: Arc<Mutex<Account>>,
    #[serde(default)]
    accounts: Arc<Mutex<Vec<Account>>>,
    #[serde(skip)]
    account_win: Arc<Mutex<AccountWin>>,
    #[serde(skip)]
    account_win_show: Arc<AtomicBool>,

    #[serde(skip)]
    instance: Arc<Mutex<Instance>>,
    #[serde(skip)]
    instance_selected: bool,
    instances: Arc<Mutex<Instances>>,
    #[serde(skip)]
    mvo: Arc<Mutex<MVOrganized>>,
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

    #[serde(skip, default = "channel::<Message>")]
    channel: (Sender<Message>, Receiver<Message>),

    #[serde(skip)]
    prog_step: Arc<AtomicUsize>,
    #[serde(skip)]
    prog_total: Arc<AtomicUsize>,

    #[serde(skip)]
    textures: Arc<Vec<egui::TextureHandle>>,
}

impl BreadLauncher {
    pub fn new(context: Context, textures: Vec<egui::TextureHandle>) -> Result<Self> {
        let client = init_reqwest()?;
        let appdir = get_appdir();
        let save = appdir.join("save.blauncher");
        let mut b = if !save.exists() {
            Self::new_clean(client.clone(), context)?
        } else {
            Self::load_launcher(appdir, context)?
        };

        b.textures = textures.into();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        let last = Duration::from_secs(b.versions_last_update);
        let ten_days = Duration::from_secs(10 * 24 * 60 * 60); // 10 days
        if ten_days <= (now - last) {
            b.mvo.lock().renew(client.clone())?;
        } else {
            b.mvo.lock().renew_version(client.clone())?;
            b.versions_last_update = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        }

        Ok(b)
    }

    fn aiw_default() -> Arc<Mutex<AddInstance>> {
        Arc::new(Mutex::new(AddInstance::default()))
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
        b.msg = Message::default();
        b.context = ctx;
        b.prog_total.fetch_add(1, Ordering::Relaxed);

        Ok(b)
    }

    fn new_clean(client: Client, ctx: Context) -> Result<Self> {
        let mut rand = [0u8; 16];
        rng().fill(&mut rand);
        let uuid = UUBuilder::from_random_bytes(rand).into_uuid().to_string();
        let instances = Instances::new();

        let b = Self {
            msg: Message::default(),
            luuid: uuid,
            versions_last_update: 0,

            about_win: Mutex::new(AboutWin {}).into(),
            about_win_show: AtomicBool::new(false).into(),

            account: Mutex::new(Account::default()).into(),
            accounts: Mutex::new(vec![]).into(),
            account_win: Mutex::new(AccountWin::default()).into(),
            account_win_show: AtomicBool::new(false).into(),

            instance: Mutex::new(Instance::default()).into(),
            instance_selected: false,
            instances: Mutex::new(instances).into(),
            mvo: Mutex::new(MVOrganized::default()).into(),
            add_instance_win: Mutex::new(AddInstance::default()).into(),
            add_instance_win_show: AtomicBool::new(false).into(),

            settings: Mutex::new(Settings::default()).into(),
            settings_win: Mutex::new(SettingsWin {}).into(),
            settings_win_show: AtomicBool::new(false).into(),

            client,
            context: ctx,

            channel: channel::<Message>(),
            prog_step: AtomicUsize::new(0).into(),
            prog_total: AtomicUsize::new(1).into(),

            textures: vec![].into(),
        };

        Ok(b)
    }

    fn show_window<T: ShowWindow + Send + 'static>(
        &self,
        ctx: &Context,
        id: impl AsRef<str>,
        win: Arc<Mutex<T>>,
        show_win: Arc<AtomicBool>,
        data: WindowData,
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
                win.lock().show(
                    mctx.clone(),
                    ctx,
                    show_win.clone(),
                    data.clone(),
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
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        let mut msg = Message::snoop();
        log::info!("Saving launcher state");
        swap(&mut msg, &mut self.msg);

        if let Err(e) = self.save_launcher() {
            log::error!("Failed to save launcher state {e}");
        }

        swap(&mut msg, &mut self.msg);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.context.forget_all_images();
    }

    fn update(&mut self, ctx: &Context, _fr: &mut Frame) {
        if let Ok(msg) = self.channel.1.try_recv() {
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

                            if ui.button("About").clicked() {
                                self.about_win_show.store(true, Ordering::Relaxed);
                            }
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
                let instance = self.instance.lock();
                ui.add(egui::Label::new(format!("Name: {}", instance.name)).wrap());
                ui.add(egui::Label::new(format!("Minecraft Version: {}", instance.mc_ver)).wrap());

                if instance.loader != InstanceLoader::Vanilla {
                    ui.add(
                        egui::Label::new(format!(
                            "{:?} Version: {}",
                            instance.loader, instance.full_ver
                        ))
                        .wrap(),
                    );
                }
            }

            ui.separator();

            ui.vertical_centered_justified(|ui| {
                // TODO enable these when implemented
                ui.disable();

                if ui.button("Add Mods").clicked() {}

                if ui.button("Logs").clicked() {}

                if ui.button("Rename").clicked() {}

                if ui.button("Delete").clicked() {}
            });

            ui.with_layout(
                egui::Layout::bottom_up(egui::Align::Center).with_cross_justify(true),
                |ui| {
                    let mut instance_lock = self.instance.lock();
                    if instance_lock.is_running() {
                        if ui.button("Stop").clicked() {
                            instance_lock.stop();
                        }

                        return;
                    }

                    if ui.button("Start Offline").clicked() {
                        if self.accounts.lock().is_empty() {
                            let _ = self
                                .channel
                                .0
                                .send(Message::msg("You have no accounts, make one"));

                            return;
                        }

                        let _ = instance_lock.run_offline(
                            self.client.clone(),
                            (self.prog_step.clone(), self.prog_total.clone()),
                            self.channel.0.clone(),
                            self.settings.lock().jvm_ram,
                            self.account.clone(),
                        );
                    }

                    if ui.button("Start/Download").clicked() {
                        if self.accounts.lock().is_empty() {
                            let _ = self
                                .channel
                                .0
                                .send(Message::msg("You have no accounts, make one"));

                            return;
                        }

                        let _ = instance_lock.run(
                            self.client.clone(),
                            (self.prog_step.clone(), self.prog_total.clone()),
                            self.channel.0.clone(),
                            self.settings.lock().jvm_ram,
                            self.account.clone(),
                        );
                    }
                },
            );
        });

        egui::TopBottomPanel::bottom("Bread Launcher - Bottom Panel (Main)").show(ctx, |ui| {
            let step = self.prog_step.load(Ordering::Relaxed);
            let total = self.prog_total.load(Ordering::Relaxed);
            let prog = step as f32 / total as f32;
            let prog = egui::ProgressBar::new(prog)
                .text(format!("{step:>4} / {total:>4}  -  {:3.2}%", prog * 100.0));
            match &self.msg {
                Message::Msg(msg) => {
                    ui.label(format!("{}: {msg}", FULLNAME));
                }
                Message::Downloading(msg) => {
                    ui.label(format!("{}: {msg}", FULLNAME));
                    ui.add(prog);
                }
                Message::Errored(msg) => {
                    ui.label(format!("{}: {msg}", FULLNAME));
                    ui.add(prog.fill(ui.style().visuals.error_fg_color));
                }
            };
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Instances");
            });

            let instances = self.instances.lock();
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
                            let selected = Arc::ptr_eq(&self.instance, instance);
                            let idx: usize = instance.lock().loader.into();
                            let icon = self.textures[idx].clone();
                            let max_img_size = [50.0, 50.0].into();

                            let widget = SelectableImageLabel::new(selected, icon, max_img_size, name);
                            let mut resp = ui.add(widget);
                            if resp.clicked() {
                                self.instance = instance.clone();
                                self.instance_selected = true;
                                resp.mark_changed();
                            }
                        }
                    });
                }

                if let Some(instances) = last {
                    ui.heading("Unnamed Group");
                    ui.separator();

                    ui.horizontal_wrapped(|ui| {
                        for (name, instance) in instances {
                            let selected = Arc::ptr_eq(&self.instance, instance);
                            let idx: usize = instance.lock().loader.into();
                            let icon = self.textures[idx].clone();
                            let max_img_size = [50.0, 50.0].into();

                            let widget = SelectableImageLabel::new(selected, icon, max_img_size, name);
                            let mut resp = ui.add(widget);
                            if resp.clicked() {
                                self.instance = instance.clone();
                                self.instance_selected = true;
                                resp.mark_changed();
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
            (self.instances.clone(), self.mvo.clone(), Arc::new(0)),
        );

        self.show_window(
            ctx,
            "Bread Launcher - Settings",
            self.settings_win.clone(),
            self.settings_win_show.clone(),
            (self.settings.clone(), Arc::new(0), Arc::new(0)),
        );

        self.show_window(
            ctx,
            "Bread Launcher - About",
            self.about_win.clone(),
            self.about_win_show.clone(),
            (Arc::new(0), Arc::new(0), Arc::new(0)),
        );

        self.show_window(
            ctx,
            "Bread Launcher - Accounts",
            self.account_win.clone(),
            self.account_win_show.clone(),
            (
                self.accounts.clone(),
                self.account.clone(),
                Arc::new(self.luuid.clone()),
            ),
        );

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.instance = Mutex::new(Instance::default()).into();
            self.instance_selected = false;
        }

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
