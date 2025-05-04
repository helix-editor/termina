// CREDIT: <https://github.com/crossterm-rs/crossterm/blob/36d95b26a26e64b0f8c12edfe11f410a6d56a812/src/event/read.rs>
// This module provides an `Arc<Mutex<T>>` wrapper around a type which is basically the crossterm
// `InternalEventReader`. This allows it to live on the Terminal and an EventStream rather than
// statically.
// Instead of crossterm's `Filter` trait I have opted for a `Fn(&Event) -> bool` for simplicity.

use std::{collections::VecDeque, io, sync::Arc, time::Duration};

use parking_lot::Mutex;

use super::{
    source::{EventSource as _, PlatformEventSource, PlatformWaker, PollTimeout},
    Event,
};

/// A reader of events from the terminal's input handle.
///
/// Note that this type wraps an `Arc` and is cheap to clone. If the `event-stream` feature is
/// enabled then this value should be passed to `EventStream::new`.
#[derive(Debug, Clone)]
pub struct EventReader {
    shared: Arc<Mutex<Shared>>,
}

impl EventReader {
    pub(crate) fn new(source: PlatformEventSource) -> Self {
        let shared = Shared {
            events: VecDeque::with_capacity(32),
            source,
            skipped_events: Vec::with_capacity(32),
        };
        Self {
            shared: Arc::new(Mutex::new(shared)),
        }
    }

    pub fn waker(&self) -> PlatformWaker {
        let reader = self.shared.lock();
        reader.source.waker()
    }

    pub fn poll<F>(&self, timeout: Option<Duration>, filter: F) -> io::Result<bool>
    where
        F: FnMut(&Event) -> bool,
    {
        let (mut reader, timeout) = if let Some(timeout) = timeout {
            let poll_timeout = PollTimeout::new(Some(timeout));
            if let Some(reader) = self.shared.try_lock_for(timeout) {
                (reader, poll_timeout.leftover())
            } else {
                return Ok(false);
            }
        } else {
            (self.shared.lock(), None)
        };
        reader.poll(timeout, filter)
    }

    pub fn read<F>(&self, filter: F) -> io::Result<Event>
    where
        F: FnMut(&Event) -> bool,
    {
        let mut reader = self.shared.lock();
        reader.read(filter)
    }
}

#[derive(Debug)]
struct Shared {
    events: VecDeque<Event>,
    source: PlatformEventSource,
    skipped_events: Vec<Event>,
}

impl Shared {
    fn poll<F>(&mut self, timeout: Option<Duration>, mut filter: F) -> io::Result<bool>
    where
        F: FnMut(&Event) -> bool,
    {
        if self.events.iter().any(&mut (filter)) {
            return Ok(true);
        }

        let timeout = PollTimeout::new(timeout);

        loop {
            let maybe_event = match self.source.try_read(timeout.leftover()) {
                Ok(None) => None,
                Ok(Some(event)) => {
                    if (filter)(&event) {
                        Some(event)
                    } else {
                        self.skipped_events.push(event);
                        None
                    }
                }
                Err(err) if err.kind() == io::ErrorKind::Interrupted => return Ok(false),
                Err(err) => return Err(err),
            };

            if timeout.elapsed() || maybe_event.is_some() {
                self.events.extend(self.skipped_events.drain(..));

                if let Some(event) = maybe_event {
                    self.events.push_front(event);
                    return Ok(true);
                }

                return Ok(false);
            }
        }
    }

    fn read<F>(&mut self, mut filter: F) -> io::Result<Event>
    where
        F: FnMut(&Event) -> bool,
    {
        let mut skipped_events = VecDeque::new();

        loop {
            while let Some(event) = self.events.pop_front() {
                if (filter)(&event) {
                    self.events.extend(skipped_events.drain(..));
                    return Ok(event);
                } else {
                    skipped_events.push_back(event);
                }
            }
            let _ = self.poll(None, &mut filter)?;
        }
    }
}
