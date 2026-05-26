use std::time::Instant;

use crate::timing::Duration;

/// Clock used by the animation runtime.
pub trait AnimationClock {
    /// Returns the current runtime timestamp.
    fn now(&self) -> Duration;
}

/// Monotonic runtime clock backed by `std::time::Instant`.
#[derive(Debug, Clone)]
pub struct SystemClock {
    started_at: Instant,
}

impl SystemClock {
    /// Creates a system clock whose zero point is now.
    #[must_use]
    pub fn new() -> Self {
        Self {
            started_at: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationClock for SystemClock {
    fn now(&self) -> Duration {
        self.started_at.elapsed().into()
    }
}
