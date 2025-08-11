# CHANGELOG

## Warning
-   Previous launcher state is absolutely incompatible with this one

## v0.0.11 -> v0.0.12

-   Removed the non-native window in the Add Instance window as it doesn't
    really do much for the clutter it made(?)
-   Changed how the icons are loaded (though it did not result in much of a
    difference in memory usage)
-   Changed the CI to also compile the rust standard lib while at it
-   Tokio is back to abort handles of spawned instances now and as a basis of
    hyper which I used as a server for oauth

# Reminders (to me)

-   To not mess with the actual contents of the data types saved in the launcher
    state to be smoothly updateable (though as this is still incomplete, this
    ain't gonna happen easily)
-   Or add contents to the data types that is not mandatory to be passed to
    serde
-   Does my build script actually add icons to the windows exe?

-   Add a case for the assets not being hashes on pre 1.6 versions
-   Figure out how to login using Microsoft accounts as the legacy and mojang
    ones (at least the ones I know the api for is not alive anymore)
-   Figured out how to align the instance icons even if the text is not
    truncated (or not)
