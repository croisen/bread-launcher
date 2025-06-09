use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub mod download;
pub mod fs;
pub mod sha1;

pub trait ShowWindow {
    fn show<T>(
        &self,
        ctx: &egui::Context,
        mctx: Arc<egui::Context>,
        data: Arc<Mutex<T>>,
        show_win: Arc<AtomicBool>,
    );
}
