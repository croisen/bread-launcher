#![allow(dead_code, unused_variables)]
#![feature(pattern)]

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
