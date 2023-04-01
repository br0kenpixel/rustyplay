use std::time::{Duration, Instant};

/// Represents a timer/countdown.
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Timer {
    /// Time at which the timer was started/created.
    start: Instant,
    /// Duration of the timer.
    len: Duration,
}

/// Methods for [`Timer`](Timer).
impl Timer {
    /// Creates a new timer with the given duration.
    pub fn new(len: Duration) -> Self {
        Timer {
            start: Instant::now(),
            len,
        }
    }

    /// Returns whether the timer has expired.
    pub fn expired(&self) -> bool {
        self.start.elapsed() >= self.len
    }
}
