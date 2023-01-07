use std::time::{Duration, Instant};

/// Represents a timer/countdown.
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Timer {
    /// Time at which the timer was started/created.
    start: Instant,
    /// Duration of the timer.
    len: Duration
}

/// Methods for [`Timer`](Timer).
impl Timer {
    /// Creates a new timer with the given duration.
    pub fn new(len: Duration) -> Self {
        Timer {
            start: Instant::now(),
            len
        }
    }

    /// Resets/Restarts the timer by setting the internal
    /// [`start`](Self::start) time to [`Instant::now()`](Instant::now())
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Similar to [`reset()`](Self::reset), but also
    /// allows changing the duration.
    pub fn rebuild(&mut self, len: Duration) {
        self.len = len;
        self.reset();
    }

    /// Returns whether the timer has expired.
    pub fn expired(&self) -> bool {
        Instant::now() - self.start >= self.len
    }
}