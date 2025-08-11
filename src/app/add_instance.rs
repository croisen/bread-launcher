use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::time::Duration;

use anyhow::bail;
use chrono::DateTime;
use egui::{Context, RichText, Ui};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::instance::{InstanceLoader, Instances};
use crate::minecraft::{MVOrganized, MinecraftVersion};
use crate::utils::ShowWindow;
use crate::utils::message::Message;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddInstance {
    name: String,
    group: String,
    release_type: &'static str,
    mc_ver: Arc<str>,
    full_ver: Arc<str>,
    version: Arc<MinecraftVersion>,
    loader: InstanceLoader,

    msg: Message,
    downloading: bool,
    step: Arc<AtomicUsize>,
    total_steps: Arc<AtomicUsize>,

    #[serde(skip, default = "AddInstance::channel_tx")]
    tx: Sender<Message>,
    #[serde(skip, default = "AddInstance::channel_rx")]
    rx: Receiver<Message>,
}

impl AddInstance {
    fn channel_tx() -> Sender<Message> {
        let (tx, _) = channel::<Message>();
        tx
    }

    fn channel_rx() -> Receiver<Message> {
        let (_, rx) = channel::<Message>();
        rx
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    fn show_vanilla(&mut self, ui: &mut Ui, mvo: Arc<dyn Any>) {
        let versions = mvo.downcast_ref::<MVOrganized>().unwrap();
        ui.vertical_centered_justified(|ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.release_type, "release", "Releases");
                ui.separator();
                ui.selectable_value(&mut self.release_type, "snapshot", "Snapshots");
                ui.separator();
                ui.selectable_value(&mut self.release_type, "old_beta", "Betas");
                ui.separator();
                ui.selectable_value(&mut self.release_type, "old_alpha", "Alphas");
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                let text = format!("{:<20} | {:<10} | {}", "Version", "Type", "Release Time");
                let wtext = RichText::new(text).monospace();
                ui.label(wtext);
                let versions = if self.release_type == "release" {
                    &versions.release
                } else if self.release_type == "snapshot" {
                    &versions.snapshot
                } else if self.release_type == "old_beta" {
                    &versions.beta
                } else if self.release_type == "old_alpha" {
                    &versions.alpha
                } else {
                    return;
                };

                for ver in versions {
                    let time = DateTime::parse_from_rfc3339(ver.release_time.as_ref())
                        .unwrap()
                        .format("%m-%d-%Y %H:%M")
                        .to_string();

                    let text = format!("{:<15} | {:<10} | {}", ver.id, ver.version_type, time);
                    let wtext = RichText::new(text).monospace();
                    if ui
                        .selectable_value(&mut self.version, ver.clone(), wtext)
                        .clicked()
                    {
                        self.mc_ver = ver.id.clone();
                    }
                }
            });
        });
    }

    fn download_vanilla(&self, cl: Client, instances: Arc<dyn Any + Send + Sync>) {
        let tx = self.tx.clone();
        let name = self.name.clone();
        let grp = self.group.clone();
        let mc_ver = self.mc_ver.clone();
        let full_ver = self.full_ver.clone();
        let version = self.version.clone();

        let step = self.step.clone();
        let total_steps = self.total_steps.clone();
        spawn(move || {
            step.store(1, Ordering::Relaxed);
            total_steps.store(2, Ordering::Relaxed);
            let _ = tx.send(Message::msg("Creating instance"));
            let e = Instances::new_vanilla_instance(cl, name, mc_ver, full_ver, version);
            if let Err(e) = &e {
                let _ = tx.send(Message::errored(format!("Instance creation failed: {e}")));
                log::error!("{e:?}");
                bail!("aaa");
            }

            step.fetch_add(1, Ordering::Relaxed);
            let _ = tx.send(Message::msg("Adding instance"));
            instances
                .downcast_ref::<Mutex<Instances>>()
                .unwrap()
                .lock()
                .unwrap()
                .add_instance(grp, e.unwrap());

            let _ = tx.send(Message::msg("Download done"));

            Ok(())
        });
    }
}

impl Default for AddInstance {
    fn default() -> Self {
        let (tx, rx) = channel::<Message>();
        Self {
            name: String::new(),
            group: String::new(),
            release_type: "release",
            mc_ver: "0".into(),
            full_ver: "0".into(),
            version: MinecraftVersion::default().into(),
            loader: InstanceLoader::Vanilla,

            msg: Message::default(),
            downloading: false,
            step: Arc::new(AtomicUsize::new(0)),
            total_steps: Arc::new(AtomicUsize::new(1)),
            tx,
            rx,
        }
    }
}

impl ShowWindow for AddInstance {
    fn show(
        &mut self,
        mctx: Context,
        ctx: &Context,
        show_win: Arc<AtomicBool>,
        instances_mutex: Arc<dyn Any + Sync + Send>,
        mvo: Arc<dyn Any + Sync + Send>,
        _: Arc<dyn Any + Sync + Send>,
        cl: Client,
    ) {
        if let Ok(msg) = self.rx.try_recv() {
            self.msg = msg;
        }

        egui::SidePanel::left("Add Instance - Side Bar").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Loaders");
                ui.separator();
                ui.selectable_value(&mut self.loader, InstanceLoader::Vanilla, "Vanilla");
                ui.selectable_value(&mut self.loader, InstanceLoader::Forge, "Forge");
                ui.selectable_value(&mut self.loader, InstanceLoader::LiteLoader, "Forgelite");
                ui.selectable_value(&mut self.loader, InstanceLoader::Fabric, "Fabric");
                ui.selectable_value(&mut self.loader, InstanceLoader::Quilt, "Quilt");
            });
        });

        egui::TopBottomPanel::bottom("Add Instance - Bottom Bar").show(ctx, |ui| {
            let msg = self.msg.clone();
            let prog = self.step.load(Ordering::Relaxed) as f64
                / self.total_steps.load(Ordering::Relaxed) as f64;
            egui::Sides::new().show(
                ui,
                |ui| {
                    ui.vertical(|ui| {
                        ui.add(egui::ProgressBar::new(prog as f32).show_percentage());
                        match msg {
                            Message::Errored(msg) => {
                                let text = egui::RichText::new(msg);
                                ui.label(text.monospace().color(ui.style().visuals.error_fg_color))
                            }
                            Message::Msg(msg) => ui.label(msg),
                            Message::Downloading(msg) => ui.label(msg),
                        };
                    });
                },
                |ui| {
                    if ui.button("Add Instance").clicked() {
                        if self.downloading {
                            return;
                        }

                        match self.loader {
                            InstanceLoader::Vanilla => {
                                self.download_vanilla(cl.clone(), instances_mutex.clone())
                            }
                            InstanceLoader::Forge => {}
                            InstanceLoader::LiteLoader => {}
                            InstanceLoader::Fabric => {}
                            InstanceLoader::Quilt => {}
                        }

                        self.downloading = true;
                    }
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                let n = ui.label("Instance Name");
                ui.text_edit_singleline(&mut self.name).labelled_by(n.id);
                let g = ui.label("Instance Group");
                ui.text_edit_singleline(&mut self.group).labelled_by(g.id);
            });

            match self.loader {
                InstanceLoader::Vanilla => self.show_vanilla(ui, mvo),
                InstanceLoader::Forge => {}
                InstanceLoader::LiteLoader => {}
                InstanceLoader::Fabric => {}
                InstanceLoader::Quilt => {}
            };
        });

        if self.msg == Message::msg("Download done") {
            self.reset();
            show_win.store(false, Ordering::Relaxed);
            mctx.request_repaint();
        }

        ctx.request_repaint_after(Duration::from_millis(50));
    }
}
