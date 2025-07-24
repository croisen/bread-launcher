#![allow(dead_code)]
#![feature(duration_constructors, mpmc_channel, pattern)]

mod app;
mod assets;
mod minecraft;
mod utils;
mod widgets;

mod account;
mod init;
mod instance;
mod logs;
mod settings;

fn main() {
    unsafe {
        #[cfg(debug_assertions)]
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(e) = app::run() {
        log::error!("{e:#?}");
    }
}
