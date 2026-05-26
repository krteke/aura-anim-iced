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

/// Deterministic clock for tests runtime checks.
#[cfg(feature = "testing")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TestClock {
    now: Duration,
}

#[cfg(feature = "testing")]
impl TestClock {
    /// Creates a test clock at zero.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            now: Duration::ZERO,
        }
    }

    /// Creates a test clock at `now`.
    #[must_use]
    pub fn at(now: impl Into<Duration>) -> Self {
        Self { now: now.into() }
    }

    /// Sets the current timestamp.
    pub fn set_now(&mut self, now: impl Into<Duration>) {
        self.now = now.into();
    }

    /// Advances the current timestamp by `duration`.
    pub fn advance_by(&mut self, duration: impl Into<Duration>) {
        self.now += duration.into();
    }
}

#[cfg(feature = "testing")]
impl Default for TestClock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "testing")]
impl AnimationClock for TestClock {
    fn now(&self) -> Duration {
        self.now
    }
}
