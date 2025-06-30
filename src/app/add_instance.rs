use std::any::Any;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use tokio::runtime::Handle;
use tokio::sync::Mutex as TKMutex;
use tokio::task::JoinHandle;

use crate::instance::{InstanceLoader, Instances};
use crate::utils::ShowWindow;
use crate::utils::message::Message;

#[derive(Debug)]
pub struct AddInstance {
    name: String,
    group: String,
    version: Arc<str>,
    release_type: String,
    loader: InstanceLoader,

    ongoing_creation: bool,
    thread: Option<JoinHandle<()>>,
    msg: Message,

    show_err_1: bool,
    show_err_2: bool,
}

impl Default for AddInstance {
    fn default() -> Self {
        Self {
            name: String::new(),
            group: String::new(),
            version: Arc::from("0"),
            release_type: "release".to_string(),
            loader: InstanceLoader::Vanilla,

            ongoing_creation: false,
            thread: None,
            msg: Message::Message("Snooping around I see".to_string()),

            show_err_1: false,
            show_err_2: false,
        }
    }
}

impl ShowWindow for AddInstance {
    fn show(
        &mut self,
        ctx: &egui::Context,
        mctx: Arc<egui::Context>,
        instances: Arc<dyn Any + Sync + Send>,
        show_win: Arc<AtomicBool>,
        appdir: impl AsRef<Path>,
        tx: Sender<Message>,
        handle: Handle,
    ) {
        egui::SidePanel::left("add-instance-side-panel").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                if ui.button("Vanilla").clicked() {
                    self.loader = InstanceLoader::Vanilla;
                    ctx.request_repaint();
                }
            });
        });

        egui::TopBottomPanel::bottom("add-instance-bottom").show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Max), |ui| {
                if ui.button("Add Instance").clicked() {
                    if self.name.len() != 0 {
                        let cctx = mctx.clone();
                        let group = self.group.clone();
                        let name = self.name.clone();
                        let ver = self.version.clone();
                        let release = self.release_type.clone();

                        let appdir = appdir.as_ref().to_path_buf();
                        let loader = self.loader;
                        let data = instances.clone();

                        let h = handle.spawn(async move {
                            let _ = tx.send(Message::Message(
                                "Creating new instance, please wait...".to_string(),
                            ));
                            if let Err(e) = data
                                .clone()
                                .downcast_ref::<TKMutex<Instances>>()
                                .unwrap()
                                .lock()
                                .await
                                .new_instance(appdir, &release, &ver.clone(), &group, &name, loader)
                                .await
                            {
                                log::error!("{e:#?}");
                                let _ = tx.send(Message::Errored(e.to_string()));
                            } else {
                                let _ = tx.send(Message::Message(
                                    "Instance creation complete".to_string(),
                                ));
                            };
                        });

                        self.thread = Some(h);
                        self.ongoing_creation = true;
                        self.name.clear();
                        self.group.clear();
                        self.version = Arc::from("0");
                        self.release_type = "release".to_string();
                        self.show_err_1 = false;
                    } else {
                        self.show_err_1 = true;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Instance Name");
            ui.text_edit_singleline(&mut self.name);
            ui.label("Instance Group");
            ui.text_edit_singleline(&mut self.group);

            ui.horizontal(|ui| {
                ui.label("Release Types: ");

                if ui.button("Release").clicked() {
                    self.release_type = "release".to_string();
                    ctx.request_repaint();
                }

                if ui.button("Snapshot").clicked() {
                    self.release_type = "snapshot".to_string();
                    ctx.request_repaint();
                }

                if ui.button("Beta").clicked() {
                    self.release_type = "old_beta".to_string();
                    ctx.request_repaint();
                }

                if ui.button("Alpha").clicked() {
                    self.release_type = "old_alpha".to_string();
                    ctx.request_repaint();
                }
            });

            egui::containers::ScrollArea::vertical().show(ui, |ui| {
                let instances = handle.block_on(
                    instances
                        .downcast_ref::<TKMutex<Instances>>()
                        .unwrap()
                        .lock(),
                );

                let versions = instances.get_versions();
                let list = if self.release_type == "release" {
                    &versions.release
                } else if self.release_type == "snapshot" {
                    &versions.snapshot
                } else if self.release_type == "old_beta" {
                    &versions.beta
                } else if self.release_type == "old_alpha" {
                    &versions.alpha
                } else {
                    &vec![]
                };

                ui.vertical_centered_justified(|ui| {
                    for ver in list {
                        ui.selectable_value(
                            &mut self.version,
                            ver.id.clone(),
                            ver.id.clone().as_ref(),
                        );
                    }
                });
            });
        });

        if let Some(thread) = &self.thread {
            if thread.is_finished() {
                self.thread = None;
                self.show_err_2 = true;
                show_win.store(false, Ordering::Relaxed);
                mctx.request_repaint();
            } else {
                self.show_err_2 = false;
            }
        }

        egui::Window::new("Bread Launcher - Add Instance: No Instance Name?")
            .title_bar(true)
            .open(&mut self.show_err_1)
            .show(ctx, |ui| {
                ui.heading("Please add a name to the instance you're trying to add, thanks");
            });

        egui::Window::new("Bread Launcher - Add Instance: Creating new Instance")
            .title_bar(false)
            .open(&mut self.show_err_2)
            .show(ctx, |ui| {
                ui.label("Creating new instance please wait...");
            });
    }
}
