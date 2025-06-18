use std::any::Any;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::mpmc::Sender;
use std::sync::Arc;

pub mod download;
pub mod fs;
pub mod message;
pub mod serde_async_mutex;
pub mod sha1;

use crate::utils::message::Message;

pub trait ShowWindow {
    fn show(
        &mut self,
        ctx: &egui::Context,
        mctx: Arc<egui::Context>,
        data: Arc<dyn Any + Sync + Send>, // tokio::sync::Mutex<T>
        show_win: Arc<AtomicBool>,
        appdir: impl AsRef<Path>,
        tx: Sender<Message>,
    );
}

pub trait ShowWindow2 {
    fn show2(
        &mut self,
        ctx: &egui::Context,
        mctx: Arc<egui::Context>,
        data1: Arc<dyn Any + Sync + Send>, // tokio::sync::Mutex<T>
        data2: Arc<dyn Any + Sync + Send>, // tokio::sync::Mutex<T>
        show_win: Arc<AtomicBool>,
        appdir: impl AsRef<Path>,
        tx: Sender<Message>,
    );
}
