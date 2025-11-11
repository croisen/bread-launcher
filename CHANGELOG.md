# CHANGELOG

## Warning
-   Previous launcher state is absolutely incompatible with this one

## v0.0.13 -> v0.0.14

-   Changed the repository's file structure a little, it wouldn't affect the
    runtime of the app that much
-   Added the urls used to list down minecraft, forge, and fabric versions

# Reminders (to me)

-   Figure out or create an API to download the mod loaders and run them with
    the vanilla client (I got a lead now eh)
-   Get an OAuth client token from Microsoft?
-   Attach the instance's child process' stdout and stderr to somewhere
    that can be used for "See Logs" in the GUI
-   Version lists could be an unsafe mutable arc later on when refreshing
    versions as it mostly is read only or just leave it to parking lot's mutex
