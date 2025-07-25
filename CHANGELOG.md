# CHANGELOG

## v0.0.9 -> v0.0.10

-   Ditch async and use the blocking stuff instead, and defer blocking tasks
    onto other threads, (man this is gonna be a massive change, for me at least)
-   While I spawned the blocking downloads in another thread, it's still
    blocking the main GUI (when done in the deferred viewport)
-   Reduced the binary size by having a longer compile time (thanks rust)
-   Updated the dependencies to match the current MSRV (1.85) though with rustup
    I and the CI using 1.88.0-nightly
-   Added a progress bar when it's downloading stuff when using the 'Start'
    function
-   The native libraries aren't extracted into the minecraft\_cache directory
    anymore, it's on the instance directory now
-   And minecraft\_cache was returned to cache
-   The deferred viewports are actually working correctly, it was me who's
    using them wrong with a long function that requires a long lock time
-   Can parse most versions now versions now (Tested the each of the oldest in
    the release, beta, and alpha versions)
-   Figured out why the instance icons are not on the same line, but it will not
    be on the same line anymore if the text is not truncated

# Reminders (to me)

-   Automatically close the add instance window if it's done and not close
    if it encounters an error
-   Do something with the 'Settings' and 'Account' windows
-   Maybe add an optional thread handle to the instances to see if it's
    currently running or not so that it can be stopped by the gui
-   Add a case for the assets not being hashes on pre 1.6 versions
