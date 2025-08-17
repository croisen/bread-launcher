use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::Context;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::init::VERSION;
use crate::utils::{ShowWindow, WindowData};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct AboutWin;

impl ShowWindow for AboutWin {
    fn show(
        &mut self,
        _mctx: Context,
        ctx: &Context,
        _show_win: Arc<AtomicBool>,
        _data: WindowData,
        _cl: Client,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.heading("Bread Launcher");
                ui.label(format!("v{VERSION}"));
                ui.separator();
                ui.hyperlink_to(
                    "Source on GitHub",
                    "https://github.com/croisen/bread-launcher",
                );
            });
        });
    }
}
