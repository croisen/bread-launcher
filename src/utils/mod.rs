use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use egui::Context;

pub mod download;
pub mod fs;
pub mod message;
pub mod serde_async_mutex;
pub mod sha1;

pub trait ShowWindow {
    // I'll use any for an async mutex
    fn show(&mut self, mctx: Context, ctx: &Context, show_win: Arc<AtomicBool>, data: Arc<dyn Any>);
}
