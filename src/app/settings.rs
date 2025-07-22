use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::Context;
use serde::{Deserialize, Serialize};
use tokio::runtime::Handle;
use tokio::sync::Mutex;

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
        data: Arc<dyn Any + Sync + Send>,
        handle: Handle,
    ) {
    }
}
