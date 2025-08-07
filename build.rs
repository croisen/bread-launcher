use std::io::Result;

#[cfg(windows)]
use winres::WindowsResource;

fn main() -> Result<()> {
    #[cfg(windows)]
    WindowsResource::new()
        .set_icon("src/assets/0.png")
        .compile()?;

    Ok(())
}
