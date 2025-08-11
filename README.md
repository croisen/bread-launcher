# Bread Launcher

To get out of the drama of other launchers having malware and being shady
I created another shady launcher that I'm the only user of (or maybe not)

I've done something wrong with the rust version and it eats up all the ram when
compiling so here we are with cpp (might take a while to sort out dependencies
but thanks to the experiments with rust, I have some idea of what I'm doing)

## Inspirations

-   [UltimMC](https://github.com/UltimMC/Launcher)
-   [iiipyuk's minecraft-launcher](https://git.a2s.su/iiiypuk/minecraft-launcher)

## TODOs

-   [ ] Automatically download the jre
-   [ ] Launching minecraft offline
    -   [ ] Version manifest
    -   [ ] Client json
    -   [ ] Libraries and client
    -   [ ] Assets
    -   [ ] Runtime arguments
    -   [ ] Launch
-   [ ] GUI
    -   [ ] Add instances
    -   [ ] Add accounts
        -   [ ] Offline Accounts
        -   [ ] Online Legacy (Address unavailable)
        -   [ ] Online Mojang (Address unavailable)
        -   [ ] Online Microsoft (OAuth seems like a pain)
        -   [ ] User properties argument
    -   [ ] Settings
    -   [ ] Instances
        -   [ ] Opening it's directory
        -   [ ] Renaming
        -   [ ] Deletion
        -   [ ] Changing loaders
        -   [ ] Adding mods
        -   [ ] Seeing logs
    -   [ ] Add a central way of adding the static icons and dynamic icons
-   [ ] Properly stop a running instance
    -   [ ] When it's still downloading game files, without hanging the gui
    -   [ ] When it had already spawned the minecraft process
-   [ ] Figure out how accounts work
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
