use crate::timing::Duration;

/// Runtime policy for animation motion behavior.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionPolicy {
    reduced_motion: bool,
    tick_interval: Duration,
}

impl MotionPolicy {
    /// Creates a motion policy.
    #[must_use]
    pub const fn new(reduced_motion: bool, tick_interval: Duration) -> Self {
        Self {
            reduced_motion,
            tick_interval,
        }
    }

    /// Returns whether reduced motion is enabled.
    #[must_use]
    pub const fn reduced_motion(self) -> bool {
        self.reduced_motion
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

impl Default for MotionPolicy {
    fn default() -> Self {
        Self {
            reduced_motion: false,
            tick_interval: Self::default_tick_interval(),
        }
    }
}
