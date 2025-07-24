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
mod settings;

fn main() {
    #[cfg(debug_assertions)]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    if let Err(e) = app::run() {
        eprintln!("{e:#?}");
    }
}
