use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::Context;
use reqwest::blocking::Client;

pub mod download;
pub mod fs;
pub mod message;
pub mod sha1;

pub trait ShowWindow {
    // Just a mutex or an unused var tbh
    fn show(
        &mut self,
        mctx: Context,
        ctx: &Context,
        show_win: Arc<AtomicBool>,
        data1: Arc<dyn Any + Sync + Send>,
        data2: Arc<dyn Any + Sync + Send>,
        cl: Client,
    );
}
