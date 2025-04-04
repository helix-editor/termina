#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

use std::{io, time::Duration};

#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
pub use windows::*;

use crate::EventStream;

/// An alias to the terminal available for the current platform.
///
/// On Windows this uses the `WindowsTerminal`, otherwise `UnixTerminal`.
#[cfg(unix)]
pub type PlatformTerminal = UnixTerminal;
#[cfg(windows)]
pub type PlatformTerminal = WindowsTerminal;

pub trait Terminal: io::Write {
    /// Enters the "raw" terminal mode.
    ///
    /// While in "raw" mode a terminal will not attempt to do any helpful interpretation of input
    /// such as waiting for Enter key presses to pass input. This is essentially the opposite of
    /// "cooked" mode. To exit raw mode, use `Self::enter_cooked_mode`.
    fn enter_raw_mode(&mut self) -> io::Result<()>;
    /// Enters the "cooked" terminal mode.
    ///
    /// This is considered the normal mode for a terminal device.
    ///
    /// While in "cooked" mode a terminal will interpret the incoming data in ways that are useful
    /// such as waiting for an Enter key press to pass input to the application.
    fn enter_cooked_mode(&mut self) -> io::Result<()>;
    fn get_dimensions(&mut self) -> io::Result<(u16, u16)>;
    fn event_stream(&self) -> EventStream;
    /// Checks if there is an `Event` available.
    ///
    /// Returns `Ok(true)` if an `Event` is available or `Ok(false)` if one is not available.
    /// If `timeout` is `None` then `poll` will block indefinitely.
    fn poll(&self, timeout: Option<Duration>) -> io::Result<bool>;
    /// Reads a single `Event` from the terminal.
    ///
    /// This function blocks until an `Event` is available. Use `poll` first to guarantee that the
    /// read won't block.
    fn read(&self) -> io::Result<crate::Event>;
}
