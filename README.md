# Bread Launcher

To get out of the drama of other launchers having malware and being shady
I created another shady launcher that I'm the only user of (or maybe not)

Java is automatically downloaded by this launcher

To see how much this application is done open the [TODOs](TODOS.md)

## Inspirations

-   [UltimMC](https://github.com/UltimMC/Launcher)
-   [iiipyuk's minecraft-launcher](https://git.a2s.su/iiiypuk/minecraft-launcher)
-   [MultiMC](https://github.com/MultiMC/Launcher)

## Build Instructions

Install rust by following [this](https://www.rust-lang.org/tools/install) or via
the package manager if you're using a Linux distribution (this uses a fairly new
rust version and nightly features)

```sh

# To build it run
cargo build --release
```

```sh
# The executable is gonna be inside the target/release folder
# run it like this?
./target/release/bread-launcher
```

## Tested Versions

-   Vanilla
    -   rd-13221 - The oldest one, launches
    -   alpha 1.0.4 - Doesn't launch, somehow the minecraft main class doesn't
        exist
    -   beta 1.0 - Launches
    -   1.0 - Launches
    -   1.2.5 - Launches, no sound tho,
    -   1.6.4 - Launches, no sound tho
    -   1.7.10 - Launches
    -   1.12.2 - Launches
    -   1.20.5 - Launches
    -   1.21.5 - Launches
-   Forge
    - None
-   Fabric
    - None
-   Liteloader
    - None
-   Neoforge
    - None
