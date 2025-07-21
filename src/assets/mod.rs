// Replace the forward slashes with two backward slashes when compiling in windows
// I'm using mingw so this works fine but for msvc this won't work well

macro_rules! add_image {
    ($x: expr) => {
        (concat!("bytes://", $x), include_bytes!($x))
    };
}

pub static ICONS: &[(&str, &[u8])] = &[add_image!("icons/0-mc-logo.png")];
