# CHANGELOG

## v0.0.8 -> v0.0.9

-   Added this CHANGELOG.md file
-   Added an iced branch but eh this seems more like a place to rewrite the egui
    parts
-   Rewrote the minecraft api to be more low level than just one central
    download function
-   Instead of having separate version folders for minecraft in the cache
    it might be possible to have one central folder that holds it all and have
    the game point to the instance folder as the game directory instead
-   The 'Start' button will attempt to download and check the hashes of the
    game files before launching while the 'Start Offline' button just launches
    the game
-   Directory Structure now looks like this:
    -   Bread Launcher (Root)
        -   instances/uuid-v7 directories
        -   java/{ver}/...
        -   logs/*.log
        -   minecraft_cache/{every_version will be stored here and not separate}
        -   temp/place_for_java_temurin_archive.zip
        -   save.blauncher
        -   save.ron
        -   version\_manifest\_v2.json

## v0.0.7 -> v0.0.8

-   Added github CI to automatically compiled tagged commits
-   Uses a cache and instances model(?) where we just copy paste an already
    downloaded versison into the instances folder
-   Croisen is the only playername for now (idk how the account works)
-   Considering to use iced as it has inherent support for async
-   Directory Structure now looks like this:
    -   Bread Launcher (Root)
        -   cache/vanilla-mc-versions
        -   instances/uuid-v7 directories
        -   java/{ver}/...
        -   logs/*.log
        -   temp/place_for_java_temurin_archive.zip
        -   save.blauncher
        -   save.ron
        -   version\_manifest\_v2.json
