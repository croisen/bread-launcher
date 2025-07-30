use std::any::Any;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use egui::Context;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::init::{get_appdir, get_cachedir, get_instancedir, get_javadir};
use crate::settings::Settings;
use crate::utils::ShowWindow;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SettingsWin;

impl ShowWindow for SettingsWin {
    fn show(
        &mut self,
        _mctx: Context,
        ctx: &Context,
        _show_win: Arc<AtomicBool>,
        settings: Arc<dyn Any + Sync + Send>,
        _: Arc<dyn Any + Sync + Send>,
        _: Arc<dyn Any + Sync + Send>,
        _cl: Client,
    ) {
        let mut system = System::new();
        system.refresh_memory();
        let max_ram = system.total_memory() as usize / (1024 * 1024);
        let mut settings = settings
            .downcast_ref::<Mutex<Settings>>()
            .unwrap()
            .lock()
            .unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Settings");
                ui.separator();

                let appdir = egui::RichText::new(format!("App Directory: {:>30?}", get_appdir()));
                let cachedir =
                    egui::RichText::new(format!("Cache Directory: {:>30?}", get_cachedir()));
                let instancedir =
                    egui::RichText::new(format!("Instance Directorry: {:>30?}", get_instancedir()));
                let javadir =
                    egui::RichText::new(format!("Java Directory: {:>30?}", get_javadir()));

                ui.label(appdir.monospace());
                ui.label(cachedir.monospace());
                ui.label(instancedir.monospace());
                ui.label(javadir.monospace());

                ui.separator();
            });

            ui.horizontal_top(|ui| {
                ui.add(
                    egui::Slider::new(&mut settings.jvm_ram, 0..=max_ram)
                        .trailing_fill(true)
                        .text("JVM ram (in MB)"),
                );
            });
        });
    }
}
