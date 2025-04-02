use rustix::termios::{self, Termios};
use std::{
    fs,
    io::{self, IsTerminal as _},
    os::unix::prelude::*,
};

use crate::EventSource;

use super::Terminal;

#[derive(Debug)]
pub(crate) enum FileDescriptor {
    Owned(OwnedFd),
    Borrowed(BorrowedFd<'static>),
}

impl AsFd for FileDescriptor {
    fn as_fd(&self) -> BorrowedFd<'_> {
        match self {
            Self::Owned(fd) => fd.as_fd(),
            Self::Borrowed(fd) => *fd,
        }
    }
}

impl FileDescriptor {
    pub const STDIN: Self = Self::Borrowed(rustix::stdio::stdin());
    pub const STDOUT: Self = Self::Borrowed(rustix::stdio::stdout());

    fn try_clone(&self) -> io::Result<Self> {
        let this = match self {
            Self::Owned(fd) => Self::Owned(fd.try_clone()?),
            Self::Borrowed(fd) => Self::Borrowed(*fd),
        };
        Ok(this)
    }
}

impl io::Read for FileDescriptor {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = rustix::io::read(&self, buf)?;
        Ok(read)
    }
}

impl io::Write for FileDescriptor {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = rustix::io::write(self, buf)?;
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn open_pty() -> io::Result<(FileDescriptor, FileDescriptor)> {
    let (read, write) = if io::stdin().is_terminal() {
        (FileDescriptor::STDIN, FileDescriptor::STDOUT)
    } else {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?;
        let read = FileDescriptor::Owned(file.into());
        let write = read.try_clone()?;
        (read, write)
    };

    // Activate non-blocking mode for the reader:
    rustix::io::ioctl_fionbio(&read, true)?;

    Ok((read, write))
}

#[derive(Debug)]
pub struct UnixTerminal {
    // Read and write handles to stdin/stdout or `/dev/tty`
    read: FileDescriptor,
    // TODO: buffer this writer.
    write: FileDescriptor,
    /// The termios of the PTY's writer detected during `Self::new`.
    original_termios: Termios,
    // parser...
}

impl UnixTerminal {
    pub fn new() -> io::Result<Self> {
        let (read, write) = open_pty()?;
        let original_termios = termios::tcgetattr(&write)?;

        Ok(Self {
            read,
            write,
            original_termios,
        })
    }

    pub fn event_source(&self) -> io::Result<EventSource> {
        EventSource::new(self.read.try_clone()?, self.write.try_clone()?)
    }
}

impl Terminal for UnixTerminal {
    fn enter_raw_mode(&mut self) -> io::Result<()> {
        let mut termios = termios::tcgetattr(&self.write)?;
        termios.make_raw();
        termios::tcsetattr(&self.write, termios::OptionalActions::Flush, &termios)?;

        // TODO: enable bracketed paste, mouse capture, etc..? Or let the consuming application do
        // so?

        Ok(())
    }

    fn exit_raw_mode(&mut self) -> io::Result<()> {
        termios::tcsetattr(
            &self.write,
            termios::OptionalActions::Now,
            &self.original_termios,
        )?;
        Ok(())
    }

    fn enter_alternate_screen(&mut self) -> std::io::Result<()> {
        // TODO: need escape sequences here.
        todo!()
    }

    fn exit_alternate_screen(&mut self) -> std::io::Result<()> {
        todo!()
    }
}

impl Drop for UnixTerminal {
    fn drop(&mut self) {
        // TODO: make the cursor visible.
        self.exit_alternate_screen().unwrap();
        // TODO: reset any bracketed paste, mouse capture, etc. that has been enabled.
        termios::tcsetattr(
            &self.write,
            termios::OptionalActions::Now,
            &self.original_termios,
        )
        .expect("failed to restore termios state");
    }
}
