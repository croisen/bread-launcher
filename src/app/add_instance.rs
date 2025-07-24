use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpmc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use anyhow::bail;
use chrono::DateTime;
use egui::{Context, RichText, Ui};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::init::init_appdir;
use crate::instance::{InstanceLoader, Instances};
use crate::utils::ShowWindow;
use crate::utils::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddInstance {
    name: String,
    group: String,
    version: Arc<str>,
    release_type: &'static str,
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

    fn show_vanilla(&mut self, ui: &mut Ui, data: Arc<dyn Any>) {
        let instances = data
            .downcast_ref::<Mutex<Instances>>()
            .unwrap()
            .lock()
            .unwrap();

        let versions = instances.get_versions();
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
                    ui.selectable_value(&mut self.version, ver.id.clone(), wtext);
                }
            });
        });
    }

    fn download_vanilla(&mut self, instances: Arc<dyn Any + Send + Sync>) {
        let tx = self.tx.clone();
        let rel_type = self.release_type;
        let ver = self.version.clone();
        let grp = self.group.clone();
        let name = self.name.clone();
        let load = self.loader;

        let step = self.step.clone();
        let total_steps = self.total_steps.clone();
        spawn(move || {
            let appdir = init_appdir()?;
            step.store(0, Ordering::Relaxed);
            total_steps.store(1, Ordering::Relaxed);
            let _ = tx.send(Message::Downloading("client.json".to_string()));
            let e = instances
                .downcast_ref::<Mutex<Instances>>()
                .unwrap()
                .lock()
                .unwrap()
                .new_instance(appdir, rel_type, ver, grp, name, load);

            if let Err(e) = &e {
                let _ = tx.send(Message::Errored(format!("Instance creation failed: {e}")));
                bail!("aaa");
            }

            step.fetch_add(1, Ordering::Relaxed);
            let _ = tx.send(Message::Message("Finished instance creation".to_string()));
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
            version: Arc::from("0"),
            release_type: "release",
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
        _mctx: Context,
        ctx: &Context,
        _show_win: Arc<AtomicBool>,
        data: Arc<dyn Any + Sync + Send>,
        _cl: Client,
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
                        InstanceLoader::Vanilla => self.download_vanilla(data.clone()),
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
                InstanceLoader::Vanilla => self.show_vanilla(ui, data.clone()),
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
    }
}
