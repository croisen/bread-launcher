pub fn load_versions() -> &'static [u8] {
    include_bytes!("versions.json")
}
