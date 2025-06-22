use std::any::Any;
use std::borrow::Cow;
use std::fs::{read, write};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use egui::load::Bytes;
use egui::ImageSource;
use flate2::write::{ZlibDecoder, ZlibEncoder};
use flate2::{Compress, Compression, Decompress};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Serializer};
use tokio::runtime::Handle;
use tokio::sync::Mutex as TKMutex;

mod add_instance;
mod widget_instance;

use crate::app::add_instance::AddInstance;
use crate::app::widget_instance::widget_instance_button;
use crate::assets::ICON_0;
use crate::instance::{Instance, Instances, UNGROUPED_NAME};
use crate::utils::message::Message;
use crate::utils::ShowWindow;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BreadLauncher {
    msg: Message,
    msg_cycles: u8, // Just to make the messages last longer?
    appdir: Arc<PathBuf>,
    last_check: u64,

    // TODO: Make a dedicated account struct later that will be passed
    // on to the run function of the Instance struct
    account: String,
    accounts: Vec<String>,
    instance: Arc<Instance>,
    #[serde(with = "crate::utils::serde_async_mutex")]
    instances: Arc<TKMutex<Instances>>,
    instance_selected: bool,

    #[serde(skip)]
    add_instance_win: Arc<Mutex<AddInstance>>,
    #[serde(skip)]
    add_instance_show: Arc<AtomicBool>,

    #[serde(skip)]
    cl: Client,
    #[serde(skip)]
    handle: Option<Handle>,

    #[serde(skip, default = "channel_2")]
    channels: (Sender<Message>, Arc<Receiver<Message>>),
}

impl BreadLauncher {
    pub fn new(cl: Client, appdir: impl AsRef<Path>, handle: Handle) -> Result<Self> {
        let f = appdir.as_ref().join("save.blauncher");
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
        if f.is_file() {
            log::info!("Config file found {f:?}");
            let saved = read(&f)?;
            let mut decomp = vec![];
            let mut d = ZlibDecoder::new_with_decompress(&mut decomp, Decompress::new(true));
            d.write_all(saved.as_ref())?;
            let _ = d.finish()?;

            let mut de = Deserializer::from_slice(decomp.as_ref());
            let mut s = Self::deserialize(&mut de)?;

            // Re-parse or re-download the version manifest if it's old
            {
                // Re-download version manifest if 10 days has passed
                let r = Duration::new(10 * 24 * 60 * 60, 0);
                let since = now.saturating_sub(Duration::new(s.last_check, 0));
                let vm = appdir.as_ref().join("version_manifest_v2.json");
                let mut instance_mutex = handle.block_on(s.instances.lock());
                if since.as_secs() >= r.as_secs() || !vm.exists() {
                    log::info!("Checking for new minecraft versions");
                    handle.block_on(instance_mutex.renew_version(appdir.as_ref()))?;
                } else {
                    handle.block_on(instance_mutex.parse_versions(appdir.as_ref()))?;
                }
            }

            s.msg_cycles = 0;
            s.instance_selected = false;
            s.cl = cl.clone();
            s.handle = Some(handle.clone());
            log::info!("Now launching");

            Ok(s)
        } else {
            log::info!("Config file not found, making default one after the app exits");
            log::info!("Creating instances collection");
            let i = handle.block_on(Instances::new(cl.clone(), appdir.as_ref()))?;
            log::info!("Now launching");
            Ok(Self {
                account: String::default(),
                accounts: vec![],
                instance: Arc::new(Instance::default()),
                instances: Arc::new(TKMutex::new(i)),
                instance_selected: false,

                appdir: Arc::new(appdir.as_ref().to_path_buf()),
                last_check: now.as_secs(),
                msg_cycles: 0,
                msg: Message::Message("Sneaking around I see".to_string()),

                add_instance_win: Arc::new(Mutex::new(AddInstance::default())),
                add_instance_show: Arc::new(AtomicBool::new(false)),

                cl,
                handle: Some(handle),
                channels: channel_2(),
            })
        }
    }

    pub fn savefile(&self) -> Result<()> {
        let mut s = vec![];
        let mut comp = vec![];
        let mut se = Serializer::pretty(&mut s);
        self.serialize(&mut se)?;

        let mut c =
            ZlibEncoder::new_with_compress(&mut comp, Compress::new(Compression::best(), true));
        let _ = c.write_all(s.as_ref())?;
        let _ = c.finish()?;

        write(self.appdir.join("save.blauncher"), comp)?;
        Ok(())
    }

    fn show_window<W: ShowWindow + Send + Sync + 'static>(
        &self,
        ctx: egui::Context,
        id: &str,
        win: Arc<Mutex<W>>,
        data: Arc<dyn Any + Send + Sync>,
        show_window: Arc<AtomicBool>,
    ) {
        if show_window.load(Ordering::Relaxed) {
            let mctx = Arc::new(ctx.clone());
            let appdir = self.appdir.clone();
            let tx = self.channels.0.clone();
            let handle = self.handle.as_ref().unwrap().clone();
            ctx.show_viewport_deferred(
                egui::ViewportId::from_hash_of(id),
                egui::ViewportBuilder::default().with_title(id),
                move |wctx, cls| {
                    assert!(
                        cls == egui::ViewportClass::Deferred,
                        "The backend doesn't support multiple viewports?"
                    );

                    win.lock().unwrap().show(
                        wctx,
                        mctx.clone(),
                        data.clone(),
                        show_window.clone(),
                        appdir.clone().as_ref(),
                        tx.clone(),
                        handle.clone(),
                    );

                    if wctx.input(|i| i.viewport().close_requested()) {
                        show_window.store(false, Ordering::Relaxed)
                    }
                },
            );
        }
    }
}

impl eframe::App for BreadLauncher {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        log::info!("Saving egui state to {:?}", self.appdir.join("egui.ron"));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let f = self.appdir.join("save.blauncher");
        log::info!("Saving app state to {:?}", f);
        self.msg = Message::Message("Sneaking around I see".to_string());
        self.msg_cycles = 0;
        self.instance = Arc::new(Instance::default());
        self.instance_selected = false;

        if let Err(e) = self.savefile() {
            log::error!("Error in saving app state to {:?}:\n\t{e:#?}", f);
        }
    }

    fn update(&mut self, ctx: &egui::Context, fr: &mut eframe::Frame) {
        let msg = self.channels.1.try_recv();
        egui::TopBottomPanel::top("main-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Add Instance").clicked() {
                    self.add_instance_show.store(true, Ordering::Relaxed);
                }

                if ui.button("About").clicked() {}
            });
        });

        egui::SidePanel::right("main-side-panel").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                if !self.instance_selected {
                    ui.disable();
                }

                if ui.button("Instance Info").clicked() {}
                if ui.button("Add Mods").clicked() {}
                if ui.button("Logs").clicked() {}
                if ui.button("Delete").clicked() {}

                ui.with_layout(egui::Layout::left_to_right(egui::Align::BOTTOM), |ui| {
                    if ui.button("Start").clicked() {
                        let instance = self.instance.clone();
                        let tx = self.channels.0.clone();
                        let _h = self.handle.as_ref().unwrap().spawn(async move {
                            if let Err(e) = instance
                                .run(
                                    "2048M".to_string(),
                                    "Croisen".to_string(),
                                    "0".to_string(),
                                    "{}".to_string(),
                                )
                                .await
                            {
                                log::error!("{e}");
                                let _ = tx.clone().send(Message::Errored(e.to_string()));
                            }
                        });
                    }
                    if ui.button("Start Offline").clicked() {}
                });
            });
        });

        egui::TopBottomPanel::bottom("main-bot-bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if self.msg_cycles == 1 {
                    ui.label(format!(
                        "bread-launcher: v{} - {:?}",
                        env!("CARGO_PKG_VERSION"),
                        self.msg
                    ));
                } else {
                    ui.label(format!(
                        "bread-launcher: v{} - Nothing much happening right now",
                        env!("CARGO_PKG_VERSION")
                    ));
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let i = self.handle.as_ref().unwrap().block_on(self.instances.lock());
            let instances = i.get_instances();
            if instances.len() == 0 {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center).with_cross_align(egui::Align::Center), |ui| {
                    ui.heading("No instance found, try adding one by using the button in the top left corner or this button");
                    if ui.button("Add Instance").clicked() {
                        self.add_instance_show.store(true, Ordering::Relaxed);
                    }
                });
            } else {
                egui::containers::ScrollArea::vertical().show(ui, |ui| {
                    let mut last = None;
                    for (k, v) in instances.iter() {
                        if k == UNGROUPED_NAME {
                            last = Some(v);
                            continue;
                        }

                        ui.vertical_centered_justified(|ui| {
                            ui.heading(k);
                        });

                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            for (n, i) in v.iter() {
                                widget_instance_button(
                                    ui,
                                    &mut self.instance,
                                    &mut self.instance_selected,
                                    i.clone(),
                                    ImageSource::Bytes {
                                        uri: Cow::Borrowed("bytes://0-mc-logo.png"),
                                        bytes: Bytes::Static(ICON_0),
                                    }, 
                                    n,
                                );
                            }
                        });
                    }

                    if let Some(last) = last {
                        ui.vertical_centered_justified(|ui| {
                            ui.heading("Ungrouped");
                        });

                        ui.separator();
                        ui.horizontal_wrapped(|ui| {
                            for (n, i) in last.iter() {
                                widget_instance_button(
                                    ui,
                                    &mut self.instance,
                                    &mut self.instance_selected,
                                    i.clone(),
                                    ImageSource::Bytes {
                                        uri: Cow::Borrowed("bytes://0-mc-logo.png"),
                                        bytes: Bytes::Static(ICON_0),
                                    }, 
                                    n,
                                );
                            }
                        });
                    }
                });
            }
        });

        self.show_window(
            ctx.clone(),
            "Bread Launcher - Add Instance",
            self.add_instance_win.clone(),
            self.instances.clone(),
            self.add_instance_show.clone(),
        );

        if let Ok(msg) = msg {
            self.msg = msg;
            self.msg_cycles = 1;
        }

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}

fn channel_2() -> (Sender<Message>, Arc<Receiver<Message>>) {
    let (tx, rx) = channel();
    (tx, Arc::new(rx))
}
