// Replace the forward slashes with two backward slashes when compiling in windows
// I'm using mingw so this works fine but for msvc this won't work well
pub static ICON_0: &[u8] = include_bytes!("icons/0-mc-logo.png");
