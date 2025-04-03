use rustix::termios::{self, Termios};
use std::{
    fs,
    io::{self, BufWriter, IsTerminal as _, Write as _},
    os::unix::prelude::*,
};

use crate::{
    event::{reader::InternalEventReader, source::UnixEventSource, InternalEvent},
    EventStream,
};

use super::Terminal;

const BUF_SIZE: usize = 4096;

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
    /// Shared wrapper around the reader (stdin or `/dev/tty`)
    reader: InternalEventReader,
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
        let reader = InternalEventReader::new(source);

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

        // TODO: enable bracketed paste, mouse capture, etc..? Or let the consuming application do
        // so?

        Ok(())
    }

    fn exit_raw_mode(&mut self) -> io::Result<()> {
        termios::tcsetattr(
            self.write.get_ref(),
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

    fn get_dimensions(&mut self) -> io::Result<(u16, u16)> {
        let winsize = termios::tcgetwinsize(self.write.get_ref())?;
        Ok((winsize.ws_row, winsize.ws_col))
    }

    fn event_stream(&self) -> EventStream {
        EventStream::new(self.reader.clone())
    }

    fn poll(&self, timeout: Option<std::time::Duration>) -> io::Result<bool> {
        self.reader
            .poll(timeout, |event| matches!(event, InternalEvent::Event(_)))
    }

    fn read(&self) -> io::Result<crate::Event> {
        self.reader
            .read(|event| matches!(event, InternalEvent::Event(_)))
            .map(|event| match event {
                InternalEvent::Event(event) => event,
                // _ => unreachable!(),
            })
    }
}

impl Drop for UnixTerminal {
    fn drop(&mut self) {
        self.exit_alternate_screen().unwrap();
        termios::tcsetattr(
            self.write.get_ref(),
            termios::OptionalActions::Now,
            &self.original_termios,
        )
        .expect("failed to restore termios state");
        self.write.flush().unwrap();
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
