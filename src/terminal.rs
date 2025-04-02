#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

use std::io;

#[cfg(unix)]
pub use unix::*;

#[cfg(windows)]
pub use windows::*;

pub trait Terminal {
    fn enter_raw_mode(&mut self) -> io::Result<()>;
    fn exit_raw_mode(&mut self) -> io::Result<()>;
    fn enter_alternate_screen(&mut self) -> io::Result<()>;
    fn exit_alternate_screen(&mut self) -> io::Result<()>;

    fn get_dimensions(&mut self) -> io::Result<(u16, u16)>;
}
