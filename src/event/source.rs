#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::time::{Duration, Instant};

#[cfg(unix)]
pub use unix::{UnixEventSource, Waker};
#[cfg(windows)]
pub use windows::{Waker, WindowsEventSource};

#[derive(Debug, Clone)]
struct PollTimeout {
    timeout: Option<Duration>,
    start: Instant,
}

impl PollTimeout {
    fn new(timeout: Option<Duration>) -> Self {
        Self {
            timeout,
            start: Instant::now(),
        }
    }

    fn leftover(&self) -> Option<Duration> {
        self.timeout.map(|timeout| {
            let elapsed = self.start.elapsed();

            if elapsed >= timeout {
                Duration::ZERO
            } else {
                timeout - elapsed
            }
        })
    }
}

pub(crate) trait EventSource: Send + Sync {
    fn try_read(
        &mut self,
        timeout: Option<Duration>,
    ) -> std::io::Result<Option<super::InternalEvent>>;

    fn waker(&self) -> Waker;
}
