# Bread Launcher

To get out of the drama of other launchers having malware and being shady
I created another shady launcher that I'm the only user of

## Tested Versions
-   1.6.4  - Doesn't launch, probably my rule parser forgot an lwjgl jar
-   1.7.10 - Doesn't launch, wasn't able to extract the native libs from a
    certain jar file (mark the rule parser and name checker for platform and
    arch)
-   1.12.2 - Doesn't launch, probably my rule parser forgot an lwjgl jar
-   1.20.5 - Launches
-   1.21.5 - Launches but crashes due to assets being in the objects folder now?

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
-   [x] Test out running the client.jar with static args (offline mode first)
-   [ ] Compile the arguments, whether it's given by the client json or not
-   [ ] Be able to launch the client jar with an offline account with the
        automatically downloaded openJDK-jre for the specific platform
-   [ ] Create a GUI
-   [ ] Add a window where one can add a specific version of minecraft as an
        isolated profile (instances, so multiple .minecraft folders)
-   [ ] Add a window to add offline accounts
-   [ ] Add a window to add online accounts
-   [ ] Figure out how to launch forge and other mod loaders
-   [ ] Add a window to modify and instance to add mods, forge, and the likes
-   [ ] Add Modrinth, Technic and other collections of modpack sources
        to the window where one can create instances
