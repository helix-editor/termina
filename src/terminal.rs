#[cfg(unix)]
mod unix;

use std::io;

#[cfg(unix)]
pub use unix::*;

pub trait Terminal {
    fn enter_raw_mode(&mut self) -> io::Result<()>;
    fn exit_raw_mode(&mut self) -> io::Result<()>;
    fn enter_alternate_screen(&mut self) -> io::Result<()>;
    fn exit_alternate_screen(&mut self) -> io::Result<()>;
}
