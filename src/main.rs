#![allow(dead_code)]
#![feature(duration_constructors, pattern)]

mod account;
mod app;
mod assets;
mod instance;
mod logs;
mod minecraft;
mod utils;

fn main() {
    if let Err(e) = app::run() {
        log::error!("{e:#?}");
    }
}
