# CHANGELOG

## v0.0.9 -> v0.0.10

-   Ditch async and use the blocking stuff instead, and defer blocking tasks
    onto other threads, (man this is gonna be a massive change, for me at least)
-   While I spawned the blocking downloads in another thread, it's still
    blocking the main GUI
-   Reduced the binary size by having a longer compile time (thanks rust)
-   Updated the dependencies to match the current MSRV (1.85) though with rustup
    I and the CI using 1.88.0-nightly
