use crate::timing::Duration;

/// Runtime policy for animation motion behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TickPolicy {
    tick_interval: Duration,
}

impl TickPolicy {
    /// Creates a tick policy.
    #[must_use]
    pub const fn new(tick_interval: Duration) -> Self {
        Self { tick_interval }
    }

    /// Returns the desired runtime tick interval.
    #[must_use]
    pub const fn tick_interval(self) -> Duration {
        self.tick_interval
    }

    /// Returns the default target tick interval for runtime-driven animation.
    #[must_use]
    pub fn default_tick_interval() -> Duration {
        Duration::from_millis(16.0)
    }
}

impl Default for TickPolicy {
    fn default() -> Self {
        Self {
            tick_interval: Self::default_tick_interval(),
        }
    }
}
