use crate::timing::Duration;

/// Product spring motion defaults.
///
/// These values describe the default spring feel used by future spring-driven
/// product motion. They are kept separate from runtime sampling so applications
/// can establish consistent motion settings before spring animation sources are
/// added.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringMotionDefaults {
    response: Duration,
    damping_ratio: f32,
    settle_epsilon: f32,
}

impl SpringMotionDefaults {
    /// Creates spring motion defaults.
    #[must_use]
    pub const fn new(response: Duration, damping_ratio: f32, settle_epsilon: f32) -> Self {
        Self {
            response,
            damping_ratio,
            settle_epsilon,
        }
    }

    /// Returns the spring response duration.
    #[must_use]
    pub const fn response(self) -> Duration {
        self.response
    }

    /// Returns the spring damping ratio.
    #[must_use]
    pub const fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }

    /// Returns the settle epsilon used by spring completion checks.
    #[must_use]
    pub const fn settle_epsilon(self) -> f32 {
        self.settle_epsilon
    }
}

impl Default for SpringMotionDefaults {
    fn default() -> Self {
        Self::new(Duration::from_millis(280.0), 0.82, 0.001)
    }
}
