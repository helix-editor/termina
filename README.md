# Termina

<!-- [![Crates.io](https://img.shields.io/crates/v/termina.svg)](https://crates.io/crates/termina) -->
<!-- [![Documentation](https://docs.rs/termina/badge.svg)](https://docs.rs/termina) -->

A cross-platform "virtual terminal" (VT) manipulation library.

Termina only "speaks text/VT" but aims to work on Windows as well as *NIX. This is made possible by Microsoft's investment into [ConPTY](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/). With some flags set via `windows-sys`, Windows consoles can be configured to speak VT like *NIX PTYs. ConPTY has been supported since Windows 10 Fall 2018 update.

API-wise Termina aims to merge the nicer parts of crossterm and termwiz. In particular termwiz's `Drop` behavior for restoring the terminal is a good idea while crossterm's `EventStream` is nice to work with. Termina tries to have a lower level API when it comes to escape sequences, however, which should make it easier to add new sequences in the future.

Termina uses significant chunks of code from both crossterm and termwiz and as such the license may be MIT (or MPL-2.0, at your option).

Currently Crossterm does not support reading VT sequences on Windows while Termwiz does. Termina will bail if the host terminal does not support VT. Note that there are some places where Termina reaches into the Windows Console API. These match Microsoft's recommendation for [exceptions for using Windows Console APIs](https://learn.microsoft.com/en-us/windows/console/classic-vs-vt#exceptions-for-using-windows-console-apis).

Termina also aims to minimize dependencies. Both Crossterm and Termwiz use the `winapi` crate which is unmaintained and superseded by the official `windows-sys` crate from Microsoft. Termina also aims to drop heavier dependencies from Crossterm like `mio` (while still maintaining macOS compatibility, thanks `rustix`).

See `examples/event-read.rs` for a look at the API.
