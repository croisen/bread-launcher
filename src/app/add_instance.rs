use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use chrono::DateTime;
use egui::{Context, RichText, Ui};
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use tokio::sync::Mutex;

use crate::instance::{InstanceLoader, Instances};
use crate::utils::ShowWindow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddInstance {
    name: String,
    group: String,
    version: Arc<str>,
    release_type: &'static str,
    loader: InstanceLoader,
}

impl AddInstance {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn show_vanilla(&mut self, ui: &mut Ui, data: Arc<dyn Any>, handle: Handle) {
        let instances = handle.block_on(data.downcast_ref::<Mutex<Instances>>().unwrap().lock());
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
}

impl Default for AddInstance {
    fn default() -> Self {
        Self {
            name: String::new(),
            group: String::new(),
            version: Arc::from("0"),
            release_type: "release",
            loader: InstanceLoader::Vanilla,
        }
    }
}

impl ShowWindow for AddInstance {
    fn show(
        &mut self,
        mctx: Context,
        ctx: &Context,
        show_win: Arc<AtomicBool>,
        data: Arc<dyn Any>,
        handle: Handle,
    ) {
        //let instances = handle.block_on(data.downcast_ref::<Mutex<Instances>>().unwrap().lock());

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
                    // Small test if it blocks the whole ui
                    let loader = self.loader;
                    handle.spawn(async move {
                        log::info!("Spawn download test");
                        tokio::time::sleep(Duration::from_secs(10)).await;
                        log::info!("Sleep complete");

                        match loader {
                            _ => {}
                        }
                    });

                    show_win.store(false, Ordering::Relaxed);
                    mctx.request_repaint();
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
                InstanceLoader::Vanilla => self.show_vanilla(ui, data.clone(), handle.clone()),
                _ => {}
            };
        });
    }
}
