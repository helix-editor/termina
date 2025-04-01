use parking_lot::Mutex;
use rustix::termios::{self, Termios};
use signal_hook::SigId;
use std::os::unix::prelude::*;
use std::{
    fs,
    io::{self, IsTerminal as _},
    os::unix::net::UnixStream,
    sync::Arc,
};

use super::Terminal;

#[derive(Debug)]
pub enum FileDescriptor {
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
    // NOTE: we use `rustix::stdio::stdin` because it gives a `BorrowedFd<'static>`.
    // `std::io::Stdin` can only be converted to a `BorrowedFd<'a>`.
    let stdin = rustix::stdio::stdin();
    let (read, write) = if stdin.is_terminal() {
        (
            FileDescriptor::Borrowed(stdin),
            FileDescriptor::Borrowed(rustix::stdio::stdout()),
        )
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
    write: FileDescriptor,
    /// The termios of the PTY's writer detected during `Self::new`.
    original_termios: Termios,
    // A pipe is registered with signal WINCH. WINCH notifies the process that the window size has
    // changed - we use this to emit an event.
    sigwinch_id: SigId,
    sigwinch_pipe: UnixStream,
    wake_pipe: UnixStream,
    wake_pipe_write: Arc<Mutex<UnixStream>>,
}

impl UnixTerminal {
    pub fn new() -> io::Result<Self> {
        let (read, write) = open_pty()?;
        let original_termios = termios::tcgetattr(&write)?;
        let (sigwinch_pipe, sigwinch_pipe_write) = UnixStream::pair()?;
        let sigwinch_id = signal_hook::low_level::pipe::register(
            // TODO: hardcode this? Will we use io_uring module elsewhere?
            rustix::io_uring::Signal::WINCH.as_raw(),
            sigwinch_pipe_write,
        )?;
        sigwinch_pipe.set_nonblocking(true)?;
        let (wake_pipe, wake_pipe_write) = UnixStream::pair()?;
        wake_pipe.set_nonblocking(true)?;
        wake_pipe_write.set_nonblocking(true)?;

        Ok(Self {
            read,
            write,
            original_termios,
            sigwinch_id,
            sigwinch_pipe,
            wake_pipe,
            wake_pipe_write: Arc::new(Mutex::new(wake_pipe_write)),
        })
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
        signal_hook::low_level::unregister(self.sigwinch_id);
        termios::tcsetattr(
            &self.write,
            termios::OptionalActions::Now,
            &self.original_termios,
        )
        .expect("failed to restore termios state");
    }
}
