macro_rules! include_icon {
    // I tried making my own str_replace that works at compile time (well, I couldn't)
    ($path: expr) => {{ (concat!("bytes://", $path), include_bytes!($path)) }};
}

pub static MINECRAFT_ICON: (&str, &[u8]) = include_icon!("0.png");
pub static FORGE_ICON: (&str, &[u8]) = include_icon!("1.png");
pub static FABRIC_ICON: (&str, &[u8]) = include_icon!("2.png");
pub static LITELOADER_ICON: (&str, &[u8]) = include_icon!("3.png");
pub static QUILT_ICON: (&str, &[u8]) = include_icon!("4.png");
