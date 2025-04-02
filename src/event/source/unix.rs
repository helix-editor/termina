use std::{
    io::{self, Read, Write as _},
    os::{fd::AsFd, unix::net::UnixStream},
    sync::Arc,
    time::Duration,
};

use parking_lot::Mutex;
use rustix::termios;
use signal_hook::SigId;

use crate::{terminal::FileDescriptor, InputEvent};

use super::PollTimeout;

#[derive(Debug)]
pub struct EventSource {
    read: FileDescriptor,
    write: FileDescriptor,
    sigwinch_id: SigId,
    sigwinch_pipe: UnixStream,
    wake_pipe: UnixStream,
    wake_pipe_write: Arc<Mutex<UnixStream>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Waker {
    inner: Arc<Mutex<UnixStream>>,
}

impl Waker {
    pub fn wake(&self) -> io::Result<()> {
        self.inner.lock().write_all(&[0])
    }
}

impl EventSource {
    pub(crate) fn new(read: FileDescriptor, write: FileDescriptor) -> io::Result<Self> {
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
            sigwinch_id,
            sigwinch_pipe,
            wake_pipe,
            wake_pipe_write: Arc::new(Mutex::new(wake_pipe_write)),
        })
    }

    pub(crate) fn waker(&self) -> Waker {
        Waker {
            inner: self.wake_pipe_write.clone(),
        }
    }

    pub fn try_read(&mut self, timeout: Option<Duration>) -> io::Result<Option<InputEvent>> {
        let timeout = PollTimeout::new(timeout);

        let mut poll_set = PollSet::new([
            PollMember::new(&self.read),
            PollMember::new(&self.sigwinch_pipe),
            PollMember::new(&self.wake_pipe),
        ]);

        while timeout.leftover().map_or(true, |t| !t.is_zero()) {
            // TODO: return buffered events from the parser.

            match poll_set.poll(timeout.leftover()) {
                Ok(_) => (),
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => return Err(err),
            }

            // The input/read pipe has data.
            if poll_set.is_ready(0) {
                todo!("read the bytes and pass them to the parser.")
            }

            // SIGWINCH received.
            if poll_set.is_ready(1) {
                // Drain the pipe.
                while read_complete(&self.wake_pipe, &mut [0; 1024])? != 0 {}

                let winsize = termios::tcgetwinsize(&self.write)?;
                let event = InputEvent::WindowResized {
                    rows: winsize.ws_row,
                    cols: winsize.ws_col,
                };
                return Ok(Some(event));
            }

            // Waker has awoken.
            if poll_set.is_ready(2) {
                // Drain the pipe.
                while read_complete(&self.wake_pipe, &mut [0; 1024])? != 0 {}
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "Poll operation was woken up",
                ));
            }
        }

        Ok(None)
    }
}

impl Drop for EventSource {
    fn drop(&mut self) {
        signal_hook::low_level::unregister(self.sigwinch_id);
    }
}

fn read_complete<F: Read>(mut file: F, buf: &mut [u8]) -> io::Result<usize> {
    loop {
        match file.read(buf) {
            Ok(read) => return Ok(read),
            Err(err) => match err.kind() {
                io::ErrorKind::WouldBlock => return Ok(0),
                io::ErrorKind::Interrupted => continue,
                _ => return Err(err),
            },
        }
    }
}

struct PollMember<'a> {
    fd: &'a dyn AsFd,
    is_ready: bool,
}

impl<'a> PollMember<'a> {
    fn new(fd: &'a dyn AsFd) -> Self {
        Self {
            fd,
            is_ready: false,
        }
    }
}

/// A small abstraction over platform specific polling behavior.
///
/// macOS `poll(2)` doesn't work on file descriptors to `/dev/tty` so we need to use `select(2)`
/// instead. This module provides a `Set` type which abstracts over the parts of `poll(2)` and
/// `select(2)` we want. Specifically we are looking for `POLLIN` events from `poll(2)` and we
/// consider that to be "ready."
///
/// This module is not meant to be generic. We consider `POLLIN` to be "ready" and do not look at
/// other poll flags. For the sake of simplicity we also only allow polling exactly three FDs at
/// a time - the exact amount we need for the event source.
struct PollSet<'a>([PollMember<'a>; 3]);

impl<'a> PollSet<'a> {
    fn new(members: [PollMember<'a>; 3]) -> Self {
        Self(members)
    }

    fn is_ready(&self, member: usize) -> bool {
        self.0[member].is_ready
    }

    fn poll(&mut self, timeout: Option<Duration>) -> std::io::Result<()> {
        let timespec = timeout.map(|timeout| timeout.try_into().unwrap());
        self.poll_impl(timespec.as_ref()).map_err(Into::into)
    }
}

#[cfg(not(target_os = "macos"))]
mod poll {
    use rustix::{
        event::{PollFd, PollFlags},
        fs::Timespec,
    };

    use super::*;

    impl PollSet<'_> {
        pub(super) fn poll_impl(&mut self, timeout: Option<&Timespec>) -> rustix::io::Result<()> {
            let mut fds = [
                PollFd::new(&self.0[0].fd, PollFlags::IN),
                PollFd::new(&self.0[1].fd, PollFlags::IN),
                PollFd::new(&self.0[2].fd, PollFlags::IN),
            ];

            rustix::event::poll(&mut fds, timeout)?;

            self.0[0].is_ready = fds[0].revents().contains(PollFlags::IN);
            self.0[1].is_ready = fds[1].revents().contains(PollFlags::IN);
            self.0[2].is_ready = fds[2].revents().contains(PollFlags::IN);

            Ok(())
        }
    }
}

#[cfg(target_os = "macos")]
mod select {
    use std::os::fd::AsRawFd;

    use rustix::{
        event::{fd_set_insert, fd_set_num_elements, FdSetElement, FdSetIter},
        fs::Timespec,
    };

    use super::*;

    impl PollSet<'_> {
        pub(super) fn poll_impl(&mut self, timeout: Option<&Timespec>) -> rustix::io::Result<()> {
            let nfds = self
                .0
                .iter()
                .map(|member| member.fd.as_fd().as_raw_fd())
                .max()
                // `self.members` is non-empty
                .unwrap()
                + 1;

            let mut readfds =
                vec![FdSetElement::default(); fd_set_num_elements(self.0.len(), nfds)];
            for member in self.0.iter() {
                fd_set_insert(&mut readfds, member.fd.as_fd().as_raw_fd());
            }

            unsafe { rustix::event::select(nfds, Some(&mut readfds), None, None, timeout) }?;

            for member in self.0.iter_mut() {
                let member_fd = member.fd.as_fd().as_raw_fd();
                if FdSetIter::new(&readfds).any(|set_fd| set_fd == member_fd) {
                    member.is_ready = true;
                }
            }

            Ok(())
        }
    }
}
