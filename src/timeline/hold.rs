use crate::timing::Duration;

/// A silent timeline segment with a fixed duration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hold {
    duration: Duration,
}

impl Hold {
    /// Creates a hold segment.
    #[must_use]
    pub fn new(duration: impl Into<Duration>) -> Self {
        Self {
            duration: duration.into(),
        }
    }

    /// Returns the hold duration.
    #[must_use]
    pub const fn total_duration(self) -> Duration {
        self.duration
    }
}
