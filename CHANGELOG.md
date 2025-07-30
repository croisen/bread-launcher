# CHANGELOG

## Warning
-   Previous launcher state is absolutely incompatible with this one (this is
    still in testing)

## v0.0.10 -> v0.0.11

-   Figured out why the instance icons are not on the same line
-   Worked around the nightly features that was used before, so that a nightly
    rust install won't be necessary for local builds
-   Stops an instance launching if the account list is empty (even if there's
    a default account that's in my name)
-   Added an optional thread handle to the instances to see if it's
    currently running or not so that it can be stopped by the gui (only for the
    running java process tho, not the download part)

# Reminders (to me)

-   Add a case for the assets not being hashes on pre 1.6 versions
-   Figure out how to login using Microsoft accounts as the legacy and mojang
    ones (at least the ones I know the api for is not alive anymore)
-   Figured out how to align the instance icons even if the text is not
    truncated (or not)
-   Figure out how to 'safely' stop a thead before it spawns a child process
