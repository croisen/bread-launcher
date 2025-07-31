# Bread Launcher

To get out of the drama of other launchers having malware and being shady
I created another shady launcher that I'm the only user of (or maybe not)

## Inspirations

-   [UltimMC](https://github.com/UltimMC/Launcher)
-   [iiipyuk's minecraft-launcher](https://git.a2s.su/iiiypuk/minecraft-launcher)

## Build Instructions

Install rust by following [this](https://www.rust-lang.org/tools/install) or via
the package manager if you're using a Linux distribution (this uses a fairly new
rust version that some Linux distributions might not have yet)

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

-   rd-13221 - The oldest one, launches
-   alpha 1.0.4 - Doesn't launch, somehow the minecraft main class doesn't exist
-   beta 1.0 - Launches
-   1.0 - Launches
-   1.2.5 - Launches, no sound tho, and it has a different way of donwloading
    the assets (pre-1.6?)
-   1.6.4 - Launches, no sound tho
-   1.7.10 - Launches
-   1.12.2 - Launches
-   1.20.5 - Launches
-   1.21.5 - Launches

## JRE Versions

-   1.20.5+         =   Java 21
-   1.17 - 1.20.4   =   Java 17
-   oldies - 1.16   =   Java 08

## TODOs

-   [x] Be able to download the libraries and the client.jar
-   [x] Use the official version manifest
-   [x] Only download the native libraries if it's a specific lib
-   [x] Try it on 'relatively' old release versions and the new release version
        as well
-   [x] Add a cmdline arg to be able to choose versions (to be removed if a GUI
        has been made)
-   [x] Download the assets
-   [x] Download Temurin-JRE {8, 17, 21} automatically (dunno where to find the
        older openJDK-JRE ones)
-   [x] Compile the arguments, whether it's given by the client json or not
-   [x] Be able to launch the client jar with an offline account with the
        automatically downloaded openJDK-jre for the specific platform
-   [x] Create a GUI
-   [x] Add a window where one can add a specific version of minecraft as an
        isolated profile (instances, so multiple .minecraft folders)
-   [x] Represent the instances with something other than plain text (I tried
        man)
-   [x] Figure out how to make a custom widget (Just a pic with a label at the
        bottom) for the instances at the main window
-   [x] Add a window to add offline accounts (It's gonna be named Croisen while
        this is not done hahahahaha)
-   [x] Add a window to add online accounts (same one above)
-   [ ] Properly stop the downloading thread and child process of an instance
        that can be done in the GUI
-   [ ] New window (even a non-native one) to modify the instances for the
    following:
    -   [ ] Renaming and Deletion
    -   [ ] Storing logs
    -   [ ] Mods even if it's a vanilla instance (just create a mod folder in
        it's instance folder)
-   [ ] Figure out how account signing in works
    -   [-] Legacy (Address unavailable)
    -   [-] Mojang (Address unavailable)
    -   [ ] Microsoft (OAuth seems like a pain to verify my new app with)
-   [ ] Launch with an online account
-   [ ] Figure out how to downlaod and launch the mod loaders
    -   [ ] Forge
    -   [ ] Forgelite
    -   [ ] Fabric
    -   [ ] Quilt
-   [ ] Add modpack support from the following sources
    -   [ ] Curseforge
    -   [ ] Modrinth
    -   [ ] FTB (hopefully)
    -   [ ] Technic
