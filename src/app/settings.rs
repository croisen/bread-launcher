use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::Context;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::settings::Settings;
use crate::utils::ShowWindow;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SettingsWin;

impl ShowWindow for SettingsWin {
    fn show(
        &mut self,
        mctx: Context,
        ctx: &Context,
        show_win: Arc<AtomicBool>,
        data1: Arc<dyn Any + Sync + Send>,
        data2: Arc<dyn Any + Sync + Send>,
        cl: Client,
    ) {
    }
}
