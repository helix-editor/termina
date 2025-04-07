use std::{io, os::windows::prelude::*, ptr, sync::Arc, time::Duration};

use windows_sys::Win32::System::Threading;

use crate::{event::Event, parse::Parser, terminal::InputHandle};

use super::{EventSource, PollTimeout};

#[derive(Debug)]
pub struct WindowsEventSource {
    input: InputHandle,
    parser: Parser,
    waker: Arc<EventHandle>,
}

impl WindowsEventSource {
    pub(crate) fn new(input: InputHandle) -> io::Result<Self> {
        Ok(Self {
            input,
            parser: Parser::default(),
            waker: Arc::new(EventHandle::new()?),
        })
    }
}

impl EventSource for WindowsEventSource {
    fn waker(&self) -> WindowsWaker {
        WindowsWaker {
            handle: self.waker.clone(),
        }
    }

    fn try_read(&mut self, timeout: Option<Duration>) -> io::Result<Option<Event>> {
        use windows_sys::Win32::Foundation::{WAIT_FAILED, WAIT_OBJECT_0};
        use Threading::{WaitForMultipleObjects, INFINITE};

        let timeout = PollTimeout::new(timeout);

        while timeout.leftover().map_or(true, |t| !t.is_zero()) {
            if let Some(event) = self.parser.pop() {
                return Ok(Some(event));
            }

            let mut pending = self.input.get_number_of_input_events()?;

            if pending == 0 {
                let mut handles = [self.input.as_raw_handle(), self.waker.as_raw_handle()];
                let wait = timeout
                    .leftover()
                    .map(|timeout| timeout.as_millis() as u32)
                    .unwrap_or(INFINITE);
                let result = unsafe {
                    WaitForMultipleObjects(handles.len() as u32, handles.as_mut_ptr(), 0, wait)
                };

                if result == WAIT_OBJECT_0 {
                    pending = self.input.get_number_of_input_events()?;
                } else if result == WAIT_OBJECT_0 + 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        "Poll operation was woken up",
                    ));
                } else if result == WAIT_FAILED {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "failed to poll input handles: {}",
                            io::Error::last_os_error()
                        ),
                    ));
                } else {
                    return Ok(None);
                }
            }

            let records = self.input.read_console_input(pending)?;

            self.parser.decode_input_records(&records);
        }

        Ok(None)
    }
}

#[derive(Debug)]
struct EventHandle {
    handle: OwnedHandle,
}

impl EventHandle {
    fn new() -> io::Result<Self> {
        let handle = unsafe { Threading::CreateEventW(ptr::null(), 0, 0, ptr::null()) };
        if handle.is_null() {
            Err(io::Error::last_os_error())
        } else {
            let handle = unsafe { OwnedHandle::from_raw_handle(handle) };
            Ok(Self { handle })
        }
    }
}

impl AsRawHandle for EventHandle {
    fn as_raw_handle(&self) -> RawHandle {
        self.handle.as_raw_handle()
    }
}

#[derive(Debug)]
pub struct WindowsWaker {
    handle: Arc<EventHandle>,
}

impl WindowsWaker {
    pub fn wake(&self) -> io::Result<()> {
        if unsafe { Threading::SetEvent(self.handle.as_raw_handle()) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}
