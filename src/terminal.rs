#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

use std::io;

#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
pub use windows::*;

use crate::EventStream;

#[cfg(unix)]
pub type PlatformTerminal = UnixTerminal;

#[cfg(windows)]
pub type PlatformTerminal = WindowsTerminal;

pub trait Terminal {
    fn enter_raw_mode(&mut self) -> io::Result<()>;
    fn exit_raw_mode(&mut self) -> io::Result<()>;
    fn enter_alternate_screen(&mut self) -> io::Result<()>;
    fn exit_alternate_screen(&mut self) -> io::Result<()>;
    fn get_dimensions(&mut self) -> io::Result<(u16, u16)>;
    fn event_stream(&self) -> io::Result<EventStream>;
}
