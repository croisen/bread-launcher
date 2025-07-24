use std::any::Any;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpmc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use anyhow::bail;
use chrono::DateTime;
use egui::{Context, RichText, Ui};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::instance::{Instance, InstanceLoader, Instances};
use crate::minecraft::{MVOrganized, Minecraft, MinecraftVersion};
use crate::utils::ShowWindow;
use crate::utils::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddInstance {
    name: String,
    group: String,
    release_type: &'static str,
    mc_ver: Arc<str>,
    full_ver: Arc<str>,
    version: Arc<MinecraftVersion>,
    loader: InstanceLoader,

    msg: Message,
    download_win_show: bool,
    step: Arc<AtomicUsize>,
    total_steps: Arc<AtomicUsize>,

    #[serde(skip, default = "AddInstance::aiw_channel_tx")]
    tx: Sender<Message>,
    #[serde(skip, default = "AddInstance::aiw_channel_rx")]
    rx: Receiver<Message>,
}

impl AddInstance {
    fn aiw_channel_tx() -> Sender<Message> {
        let (tx, _) = channel::<Message>();
        tx
    }

    fn aiw_channel_rx() -> Receiver<Message> {
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

    fn download_vanilla(&mut self, cl: Client, instances: Arc<dyn Any + Send + Sync>) {
        let tx = self.tx.clone();
        let name = self.name.clone();
        let grp = self.group.clone();
        let mc_ver = self.mc_ver.clone();
        let full_ver = self.full_ver.clone();
        let version = self.version.clone();
        let load = self.loader;

        let step = self.step.clone();
        let total_steps = self.total_steps.clone();
        spawn(move || {
            step.store(1, Ordering::Relaxed);
            total_steps.store(3, Ordering::Relaxed);
            let _ = tx.send(Message::Downloading("Downloading client.json".to_string()));
            let e = version.download(&cl);
            if let Err(e) = &e {
                let _ = tx.send(Message::Errored(format!("Instance creation failed: {e}")));
                bail!("aaa");
            }

            let e = Minecraft::new(Path::new("a"), mc_ver.as_ref());
            if let Err(e) = &e {
                let _ = tx.send(Message::Errored(format!("Instance creation failed: {e}")));
                bail!("aaa");
            }

            step.fetch_add(1, Ordering::Relaxed);
            let _ = tx.send(Message::Downloading("Downloading client.jar".to_string()));
            let e = e.unwrap().new_instance();
            if let Err(e) = &e {
                let _ = tx.send(Message::Errored(format!("Instance creation failed: {e}")));
                bail!("aaa");
            }

            let mc = e.unwrap();
            let instance = Instance::new(cl, name, mc_ver, full_ver, mc.get_cache_dir(), load);
            let instances = instances.downcast_ref::<Mutex<Instances>>().unwrap();
            step.fetch_add(1, Ordering::Relaxed);
            let _ = tx.send(Message::Downloading("Adding instance".to_string()));
            instances.lock().unwrap().add_instance(grp, instance);
            let _ = tx.send(Message::Message("Download done".to_string()));

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
            download_win_show: false,
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
                ui.selectable_value(&mut self.loader, InstanceLoader::Forgelite, "Forgelite");
                ui.selectable_value(&mut self.loader, InstanceLoader::Fabric, "Fabric");
                ui.selectable_value(&mut self.loader, InstanceLoader::Quilt, "Quilt");
            });
        });

        egui::TopBottomPanel::bottom("Add Instance - Bottom Bar").show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Add Instance").clicked() {
                    if self.download_win_show {
                        return;
                    }

                    match self.loader {
                        InstanceLoader::Vanilla => {
                            self.download_vanilla(cl.clone(), instances_mutex.clone())
                        }
                        _ => {}
                    }

                    self.download_win_show = true;
                }
            });
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
                _ => {}
            };
        });

        egui::Window::new("Downloading")
            .open(&mut self.download_win_show)
            .show(ctx, |ui| {
                let prog =
                    self.step.load(Ordering::Relaxed) / self.total_steps.load(Ordering::Relaxed);

                ui.add(egui::ProgressBar::new(prog as f32).show_percentage());
                ui.label(format!("{:?}", self.msg));
            });

        if self.msg == Message::Message("Download done".to_string()) {
            self.reset();
            show_win.store(false, Ordering::Relaxed);
            mctx.request_repaint();
        }
    }
}
