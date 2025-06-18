use std::any::Any;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpmc::Sender;
use std::sync::{Arc, Mutex};

use tokio::sync::Mutex as TKMutex;

use crate::instance::{InstanceLoader, Instances};
use crate::minecraft::MinecraftVersionManifest;
use crate::utils::message::Message;
use crate::utils::ShowWindow2;

#[derive(Debug)]
pub struct AddInstance {
    name: String,
    group: String,
    version: String,
    release_type: String,
    loader: InstanceLoader,

    show_err_1: bool,
}

impl Default for AddInstance {
    fn default() -> Self {
        Self {
            name: String::new(),
            group: String::new(),
            version: String::new(),
            release_type: "release".to_string(),
            loader: InstanceLoader::Vanilla,

            show_err_1: false,
        }
    }
}

impl ShowWindow2 for AddInstance {
    fn show2(
        &mut self,
        ctx: &egui::Context,
        mctx: Arc<egui::Context>,
        instances: Arc<dyn Any + Sync + Send>,
        versions: Arc<dyn Any + Sync + Send>,
        show_win: Arc<AtomicBool>,
        appdir: impl AsRef<Path>,
        tx: Sender<Message>,
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
                        show_win.store(false, Ordering::Relaxed);
                        let cctx = mctx.clone();
                        let name = self.name.clone();
                        let ver = self.version.clone();
                        let release = self.release_type.clone();
                        let appdir = appdir.as_ref().to_path_buf();
                        let loader = self.loader;
                        let data = instances.clone();
                        let group = if self.group.len() == 0 {
                            None
                        } else {
                            Some(self.group.clone())
                        };

                        tokio::spawn(async move {
                            let _ = tx.send(Message::Message(
                                "Creating new instance, please wait...".to_string(),
                            ));
                            if let Err(e) = data
                                .clone()
                                .downcast_ref::<TKMutex<Instances>>()
                                .unwrap()
                                .lock()
                                .await
                                .new_instance(appdir, &release, &ver, group, &name, loader)
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

                        self.name.clear();
                        self.group.clear();
                        self.version = "release".to_string();
                        self.release_type.clear();
                        self.show_err_1 = false;
                        mctx.request_repaint();
                    } else {
                        self.show_err_1 = true;
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, World!");

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

                if ui.button("Snapshots").clicked() {
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
                let versions = versions
                    .downcast_ref::<Mutex<MinecraftVersionManifest>>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .versions
                    .clone()
                    .iter()
                    .filter(|c| c.version_type.eq_ignore_ascii_case(&self.release_type))
                    .map(|c| c.id.clone())
                    .collect::<Vec<Arc<str>>>();

                ui.vertical_centered_justified(|ui| {
                    for ver in versions {
                        ui.selectable_value(
                            &mut self.version,
                            ver.to_string(),
                            ver.clone().as_ref(),
                        );
                    }
                });
            });
        });

        egui::Window::new("add-instance-error-1")
            .title_bar(true)
            .open(&mut self.show_err_1)
            .show(ctx, |ui| {
                ui.heading("Please add a name to the instance you're trying to add, thanks");
            });
    }
}
