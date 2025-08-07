// Replace the forward slashes with two backward slashes when compiling in windows
// I'm using mingw so this works fine but for msvc this won't work well (just
// don't replace, the ones inside the include_icon macro_rule

mod icons;

pub static ICONS: &[(&str, &[u8])] = &[
    icons::MINECRAFT_ICON,
    icons::FORGE_ICON,
    icons::FABRIC_ICON,
    icons::LITELOADER_ICON,
    icons::QUILT_ICON,
];
