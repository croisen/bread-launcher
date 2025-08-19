# CHANGELOG

## Warning
-   Previous launcher state is absolutely incompatible with this one

## v0.0.12 -> v0.0.13

-   I believe to have used the async functions so wrongly with tokio that the
    ram usage to compile this (v0.0.12) inflated
-   Stops the running instance with channels that are stuck on being routinely
    checked by a thread (may use rayon for this but idk how to do that yet)
-   Added a search to the Add Instance window to easily get the version you want
-   Removed some of the dependencies of my own dependencies to reduce compile
    times and file size
-   Added some of the instance management features like renaming, deletion,
    and opening it's directory

-   Changed the directory structure again to
    -   %APPDATA%\\Bread Launcher (on windows, still unchanged tho)
    -   ~/.local/share/bread-launcher (on linux, this changed)
        -   cache/{assets,libraries,versions}
        -   instances/{.minecraft,natives}  # this changed
        -   java/{versions}
        -   logs/
        -   temp/temurin.zip
        -   save.blauncher
        -   save.ron
        -   version\_manifest\_v2.json

# Reminders (to me)

-   To not mess with the actual contents of the data types saved in the launcher
    state to be smoothly updateable (though as this is still incomplete, this
    ain't gonna happen easily)
-   Or add contents to the data types that is not mandatory to be passed to
    serde
-   Does my build script actually add icons to the windows exe?

-   Figure out how to login using Microsoft accounts as the legacy and mojang
    ones (at least the ones I know the api for is not alive anymore)
-   Figure out or create an API to download the mod loaders and run them with
    the vanilla client
