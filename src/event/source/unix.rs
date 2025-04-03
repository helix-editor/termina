use std::{
    io::{self, Read, Write as _},
    os::{fd::AsFd, unix::net::UnixStream},
    sync::Arc,
    time::Duration,
};

use parking_lot::Mutex;
use rustix::termios;
use signal_hook::SigId;

use crate::{event::InternalEvent, parse::Parser, terminal::FileDescriptor, InputEvent};

use super::{EventSource, PollTimeout};

#[derive(Debug)]
pub struct UnixEventSource {
    parser: Parser,
    buffer: [u8; 1024],
    read: FileDescriptor,
    write: FileDescriptor,
    sigwinch_id: SigId,
    sigwinch_pipe: UnixStream,
    wake_pipe: UnixStream,
    wake_pipe_write: Arc<Mutex<UnixStream>>,
}

#[derive(Debug, Clone)]
pub struct UnixWaker {
    inner: Arc<Mutex<UnixStream>>,
}

impl UnixWaker {
    pub fn wake(&self) -> io::Result<()> {
        self.inner.lock().write_all(&[0])
    }
}

impl UnixEventSource {
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
            parser: Default::default(),
            buffer: [0; 1024],
            read,
            write,
            sigwinch_id,
            sigwinch_pipe,
            wake_pipe,
            wake_pipe_write: Arc::new(Mutex::new(wake_pipe_write)),
        })
    }
}

impl Drop for UnixEventSource {
    fn drop(&mut self) {
        signal_hook::low_level::unregister(self.sigwinch_id);
    }
}

impl EventSource for UnixEventSource {
    fn waker(&self) -> UnixWaker {
        UnixWaker {
            inner: self.wake_pipe_write.clone(),
        }
    }

    fn try_read(&mut self, timeout: Option<Duration>) -> io::Result<Option<InternalEvent>> {
        let timeout = PollTimeout::new(timeout);

        while timeout.leftover().map_or(true, |t| !t.is_zero()) {
            if let Some(event) = self.parser.next() {
                return Ok(Some(event));
            }

            let [read_ready, sigwinch_ready, wake_ready] = match poll(
                [&self.read, &self.sigwinch_pipe, &self.wake_pipe],
                timeout.leftover(),
            ) {
                Ok(ready) => ready,
                Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
                Err(err) => return Err(err),
            };

            // The input/read pipe has data.
            if read_ready {
                let read_count = read_complete(&mut self.read, &mut self.buffer)?;
                if read_count > 0 {
                    todo!("advance the parser");
                }
                if let Some(event) = self.parser.next() {
                    return Ok(Some(event));
                }
                if read_count == 0 {
                    break;
                }
            }

            // SIGWINCH received.
            if sigwinch_ready {
                // Drain the pipe.
                while read_complete(&self.wake_pipe, &mut [0; 1024])? != 0 {}

                let winsize = termios::tcgetwinsize(&self.write)?;
                let event = InputEvent::WindowResized {
                    rows: winsize.ws_row,
                    cols: winsize.ws_col,
                };
                return Ok(Some(InternalEvent::InputEvent(event)));
            }

            // Waker has awoken.
            if wake_ready {
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

/// A small abstraction over platform specific polling behavior.
///
/// macOS `poll(2)` doesn't work on file descriptors to `/dev/tty` so we need to use `select(2)`
/// instead. This provides a function which abstracts over the parts of `poll(2)` and
/// `select(2)` we want. Specifically we are looking for `POLLIN` events from `poll(2)` and we
/// consider that to be "ready."
///
/// This module is not meant to be generic. We consider `POLLIN` to be "ready" and do not look at
/// other poll flags. For the sake of simplicity we also only allow polling exactly three FDs at
/// a time - the exact amount we need for the event source.
fn poll(fds: [&dyn AsFd; 3], timeout: Option<Duration>) -> std::io::Result<[bool; 3]> {
    use rustix::fs::Timespec;

    #[cfg_attr(target_os = "macos", allow(dead_code))]
    fn poll2(fds: [&dyn AsFd; 3], timeout: Option<&Timespec>) -> io::Result<[bool; 3]> {
        use rustix::event::{PollFd, PollFlags};
        let mut fds = [
            PollFd::new(&fds[0], PollFlags::IN),
            PollFd::new(&fds[1], PollFlags::IN),
            PollFd::new(&fds[2], PollFlags::IN),
        ];

        rustix::event::poll(&mut fds, timeout)?;

        Ok([
            fds[0].revents().contains(PollFlags::IN),
            fds[1].revents().contains(PollFlags::IN),
            fds[2].revents().contains(PollFlags::IN),
        ])
    }

    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    fn select2(fds: [&dyn AsFd; 3], timeout: Option<&Timespec>) -> rustix::io::Result<[bool; 3]> {
        use rustix::event::{fd_set_insert, fd_set_num_elements, FdSetElement, FdSetIter};
        use std::os::fd::AsRawFd;

        let fds = [
            fds[0].as_fd().as_raw_fd(),
            fds[1].as_fd().as_raw_fd(),
            fds[2].as_fd().as_raw_fd(),
        ];
        // The array is non-empty so `max()` cannot return `None`.
        let nfds = fds.iter().copied().max().unwrap() + 1;

        let mut readfds = vec![FdSetElement::default(); fd_set_num_elements(fds.len(), nfds)];
        for fd in fds {
            fd_set_insert(&mut readfds, fd);
        }

        unsafe { rustix::event::select(nfds, Some(&mut readfds), None, None, timeout) }?;

        let mut ready = [false; 3];
        for (fd, is_ready) in fds.iter().copied().zip(ready.iter_mut()) {
            if FdSetIter::new(&readfds).any(|set_fd| set_fd == fd) {
                *is_ready = true;
            }
        }
        Ok(ready)
    }

    #[cfg(not(target_os = "macos"))]
    use poll2 as poll_impl;
    #[cfg(target_os = "macos")]
    use select2 as poll_impl;

    let timespec = timeout.map(|timeout| timeout.try_into().unwrap());
    poll_impl(fds, timespec.as_ref())
}
