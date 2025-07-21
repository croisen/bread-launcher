#![allow(dead_code)]
#![feature(duration_constructors, pattern)]

mod app;
mod assets;
mod minecraft;
mod utils;
mod widgets;

mod account;
mod instance;
mod logs;
mod settings;

fn main() {
    if let Err(e) = app::run() {
        log::error!("{e:#?}");
    }
}
