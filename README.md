# Termina

[![Crates.io](https://img.shields.io/crates/v/termina.svg)](https://crates.io/crates/termina)
[![Documentation](https://docs.rs/termina/badge.svg)](https://docs.rs/termina)

A cross-platform "virtual terminal" (VT) manipulation library.

Termina only "speaks text/VT" but aims to work on Windows as well as *NIX. This is made possible by Microsoft's investment into [ConPTY](https://devblogs.microsoft.com/commandline/windows-command-line-introducing-the-windows-pseudo-console-conpty/). This means that Termina requires 64-bit Windows 10.0.17763 (released around Fall 2018) or later ([same as WezTerm](https://wezterm.org/install/windows.html)).

Termina is a cross between [Crossterm](https://github.com/crossterm-rs/crossterm) and [TermWiz](https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/termwiz/README.md) with a lower level API which exposes escape codes to consuming applications. The aim is to scale well in the long run as terminals introduce VT extensions like the [Kitty Keyboard Protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/) or [Contour's Dark/Light mode detection](https://contour-terminal.org/vt-extensions/color-palette-update-notifications/) - those requiring minimal changes in Termina and also allowing flexibility in how applications detect and handle these extensions. See `examples/event-read.rs` for a look at a basic API.

## Credit

Termina contains significant code sourced and/or modified from other projects, especially Crossterm and TermWiz. See "CREDIT" comments in the source for details on what was copied and what modifications were made. Since all copied code is licensed under MIT, Termina is offered under the MIT license as well at your option.

## License

Licensed under either of:

 * Mozilla Public License, v. 2.0, ([LICENSE-MPL](./LICENSE-MPL) or http://mozilla.org/MPL/2.0/)
 * MIT license ([LICENSE-MIT](./LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.
