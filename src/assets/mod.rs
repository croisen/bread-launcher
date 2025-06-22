#[cfg(target_family = "windows")]
pub static ICON_0: &'static [u8] = include_bytes!("icons\\0-mc-logo.png");
#[cfg(target_family = "unix")]
pub static ICON_0: &'static [u8] = include_bytes!("icons/0-mc-logo.png");
