use rustix::termios::{self, Termios};
use std::{
    fs,
    io::{self, BufWriter, IsTerminal as _, Write as _},
    os::unix::prelude::*,
};

use crate::{
    event::{reader::EventReader, source::UnixEventSource},
    Event, EventStream,
};

use super::Terminal;

const BUF_SIZE: usize = 4096;

// CREDIT: FileDescriptor stuff is mostly based on the WezTerm crate `filedescriptor` but has been
// rewritten with `rustix` instead of `libc`.
// <https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/filedescriptor/src/unix.rs>

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

    // Activate non-blocking mode for the reader.
    // NOTE: this seems to make macOS consistently fail with io::ErrorKind::WouldBlock errors.
    // rustix::io::ioctl_fionbio(&read, true)?;

    Ok((read, write))
}

// CREDIT: <https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/termwiz/src/terminal/unix.rs>
// Some discussion, though: Termwiz's terminals combine the terminal interaction (reading
// dimensions, reading events, writing bytes, etc.) all in one type. Crossterm splits these
// concerns and I prefer that interface. As such this type is very much based on Termwiz'
// `UnixTerminal` but the responsibilities are split between this file and
// `src/event/source/unix.rs` - the latter being more inspired by crossterm.
// Ultimately this terminal doesn't look much like Termwiz' due to the use of `rustix` and
// differences in the trait and `Drop` behavior (see `super`'s CREDIT comment).

#[derive(Debug)]
pub struct UnixTerminal {
    /// Shared wrapper around the reader (stdin or `/dev/tty`)
    reader: EventReader,
    /// Buffered handle to the writer (stdout or `/dev/tty`)
    write: BufWriter<FileDescriptor>,
    /// The termios of the PTY's writer detected during `Self::new`.
    original_termios: Termios,
}

impl UnixTerminal {
    pub fn new() -> io::Result<Self> {
        let (read, write) = open_pty()?;
        let source = UnixEventSource::new(read, write.try_clone()?)?;
        let original_termios = termios::tcgetattr(&write)?;
        let reader = EventReader::new(source);

        Ok(Self {
            reader,
            write: BufWriter::with_capacity(BUF_SIZE, write),
            original_termios,
        })
    }
}

impl Terminal for UnixTerminal {
    fn enter_raw_mode(&mut self) -> io::Result<()> {
        let mut termios = termios::tcgetattr(self.write.get_ref())?;
        termios.make_raw();
        termios::tcsetattr(
            self.write.get_ref(),
            termios::OptionalActions::Flush,
            &termios,
        )?;

        Ok(())
    }

    fn enter_cooked_mode(&mut self) -> io::Result<()> {
        termios::tcsetattr(
            self.write.get_ref(),
            termios::OptionalActions::Now,
            &self.original_termios,
        )?;
        Ok(())
    }

    fn reset_mode(&mut self) -> io::Result<()> {
        // NOTE: this is the same as entering cooked mode on Unix but involves more on Windows.
        self.enter_cooked_mode()
    }

    fn get_dimensions(&self) -> io::Result<(u16, u16)> {
        let winsize = termios::tcgetwinsize(self.write.get_ref())?;
        Ok((winsize.ws_row, winsize.ws_col))
    }

    fn event_stream<F: Fn(&Event) -> bool + Clone + Send + Sync + 'static>(
        &self,
        filter: F,
    ) -> EventStream<F> {
        EventStream::new(self.reader.clone(), filter)
    }

    fn poll<F: Fn(&Event) -> bool>(
        &self,
        filter: F,
        timeout: Option<std::time::Duration>,
    ) -> io::Result<bool> {
        self.reader.poll(timeout, filter)
    }

    fn read<F: Fn(&Event) -> bool>(&self, filter: F) -> io::Result<Event> {
        self.reader.read(filter)
    }
}

impl Drop for UnixTerminal {
    fn drop(&mut self) {
        let _ = self.flush();
        let _ = self.reset_mode();
    }
}

impl io::Write for UnixTerminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.write.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.write.flush()
    }
}
