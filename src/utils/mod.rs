use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::Context;
use reqwest::blocking::Client;

pub mod download;
pub mod fs;
pub mod message;
pub mod sha1;

pub type WindowData = (
    Arc<dyn Any + Sync + Send>,
    Arc<dyn Any + Sync + Send>,
    Arc<dyn Any + Sync + Send>,
);

pub trait ShowWindow {
    // Just a mutex or an unused var tbh
    fn show(
        &mut self,
        mctx: Context,
        ctx: &Context,
        show_win: Arc<AtomicBool>,
        data: WindowData,
        cl: Client,
    );
}
