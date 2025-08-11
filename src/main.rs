#![allow(dead_code, unused_imports)]
#![cfg_attr(
    not(debug_assertions),
    cfg_attr(target_family = "windows", windows_subsystem = "windows")
)]

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
        std::env::set_var("RUST_BACKTRACE", "1")
    };

    if let Err(e) = app::launch() {
        eprintln!("{e:#?}");
    }
}
