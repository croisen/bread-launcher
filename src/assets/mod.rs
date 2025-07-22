// Replace the forward slashes with two backward slashes when compiling in windows
// I'm using mingw so this works fine but for msvc this won't work well

use egui::{ImageSource, include_image};

pub static ICONS: &[ImageSource] = &[include_image!("icons/0-mc-logo.png")];
