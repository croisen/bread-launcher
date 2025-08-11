# TODOs

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
-   [x] Add the about window that would link back to here
-   [x] Properly stop a running instance
    -   [x] When it's still downloading game files, without hanging the gui
    -   [x] When it had already spawned the minecraft process
-   [ ] New window (even a non-native one) to modify the instances for the
    following:
    -   [ ] Renaming and Deletion
    -   [ ] Storing logs
    -   [ ] Mods even if it's a vanilla instance (just create a mod folder in
        it's instance folder)
    -   [ ] Changing it's loader type (vanilla, forge, ...)
-   [ ] Add a central way of adding the static icons and dynamic icons via the
    the egui textures manager?
-   [ ] Combine types when it's being sent to other threads as clippy is
    complaining
-   [ ] Figure out how accounts work
    -   [x] Offline Accounts
    -   [ ] Online Legacy (Address unavailable)
    -   [ ] Online Mojang (Address unavailable)
    -   [ ] Online Microsoft (OAuth seems like a pain to verify my new app with)
    -   [ ] User properties argument
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
