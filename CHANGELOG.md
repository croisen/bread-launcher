# CHANGELOG

## Warning
-   Previous launcher state is absolutely incompatible with this one (this is
    still in testing)

## v0.0.11 -> v0.0.12

-   Removed the non-native window in the Add Instance window as it doesn't
    really do much for the clutter it made(?)
-   Changed how the icons are loaded (though it did not result in much of a
    difference in memory usage)

# Reminders (to me)

-   To not mess with the actual contents of the data types saved in the launcher
    state to be smoothly updateable (though as this is still incomplete, this
    ain't gonna happen easily)

-   Add a case for the assets not being hashes on pre 1.6 versions
-   Figure out how to login using Microsoft accounts as the legacy and mojang
    ones (at least the ones I know the api for is not alive anymore)
-   Figured out how to align the instance icons even if the text is not
    truncated (or not)
-   Figure out how to 'safely' stop a thead before it spawns a child process
-   Current Instance::is\_running function is wrong as the thread returns early
    when it spawns the child process
-   Store both the thread and child process to see if it's running and stop
    either of them from the GUI

-   Figure out or create an API to download the mod loaders and run them with
    the vanilla client
