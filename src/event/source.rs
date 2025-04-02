#[cfg(unix)]
mod unix;

use std::time::{Duration, Instant};

#[cfg(unix)]
pub use unix::EventSource;

#[derive(Debug, Clone)]
pub struct PollTimeout {
    timeout: Option<Duration>,
    start: Instant,
}

impl PollTimeout {
    pub fn new(timeout: Option<Duration>) -> Self {
        Self {
            timeout,
            start: Instant::now(),
        }
    }

    // pub fn elapsed(&self) -> bool {
    //     self.timeout
    //         .map(|timeout| self.start.elapsed() >= timeout)
    //         .unwrap_or(false)
    // }

    pub fn leftover(&self) -> Option<Duration> {
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
