// Replace the forward slashes with two backward slashes when compiling in windows
// I'm using mingw so this works fine but for msvc this won't work well (just
// don't replace, the ones inside the include_icon macro_rule

macro_rules! include_icon {
    // I tried making my own str_replace that works at compile time (well, I couldn't)
    ($path: expr) => {{ (concat!("bytes://", $path), include_bytes!($path)) }};
}

pub static ICONS: &[(&str, &[u8])] = &[include_icon!("icons/0-mc-logo.png")];
