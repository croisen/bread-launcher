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
-   Removed the unnecessary Arc(s) that I can see at the moment and sorta
    reduced memory usage (not like it uses much in the first place)
-   My implementation for icons now actually supports resizing (even when it had
    options to change sizes, the actual display was fixed, idk how I not noticed)
-   Guesses towards the Forge json structure or I'll make one myself redirecting
    to their ad page and getting the installer jar from the downloads folder

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

-   Does my build script actually add icons to the windows exe?

-   Figure out or create an API to download the mod loaders and run them with
    the vanilla client
-   Use actual table widgets on lists instead of formatted text
-   Get an OAuth client token from Microsoft?
-   Attach the instance's child process' stdout and stderr to somewhere
    that can be used for "See Logs" in the GUI
