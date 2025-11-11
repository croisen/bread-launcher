#![allow(dead_code)]
#![feature(mpmc_channel)] // It's back again
#![cfg_attr(
    not(debug_assertions),
    cfg_attr(target_family = "windows", windows_subsystem = "windows")
)]

mod app;
mod assets;
mod loaders;
mod utils;

mod account;
mod init;
mod instance;

#[cfg(test)]
mod tests;

fn main() {
    #[cfg(debug_assertions)]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1")
    };

    if let Err(e) = app::launch() {
        log::error!("{e:#?}");
    }
}
