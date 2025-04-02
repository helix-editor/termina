use std::{collections::VecDeque, io, sync::Arc, time::Duration};

use parking_lot::Mutex;

use super::{
    source::{EventSource as _, PlatformEventSource, PollTimeout},
    InternalEvent,
};

#[derive(Debug, Clone)]
pub struct InternalEventReader {
    shared: Arc<Mutex<Shared>>,
}

impl InternalEventReader {
    pub fn new(source: PlatformEventSource) -> Self {
        let shared = Shared {
            events: VecDeque::with_capacity(32),
            source,
            skipped_events: Vec::with_capacity(32),
        };
        Self {
            shared: Arc::new(Mutex::new(shared)),
        }
    }

    pub fn poll<F>(&self, timeout: Option<Duration>, filter: F) -> io::Result<bool>
    where
        F: Fn(&InternalEvent) -> bool,
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

    pub fn read<F>(&self, filter: F) -> io::Result<InternalEvent>
    where
        F: Fn(&InternalEvent) -> bool,
    {
        let mut reader = self.shared.lock();
        reader.read(filter)
    }
}

#[derive(Debug)]
struct Shared {
    events: VecDeque<InternalEvent>,
    source: PlatformEventSource,
    skipped_events: Vec<InternalEvent>,
}

impl Shared {
    fn poll<F>(&mut self, timeout: Option<Duration>, filter: F) -> io::Result<bool>
    where
        F: Fn(&InternalEvent) -> bool,
    {
        if self.events.iter().any(&filter) {
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

    fn read<F>(&mut self, filter: F) -> io::Result<InternalEvent>
    where
        F: Fn(&InternalEvent) -> bool,
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
            let _ = self.poll(None, &filter)?;
        }
    }
}
