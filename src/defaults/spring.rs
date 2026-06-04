use crate::{
    spring::{ScalarSpring, SpringConfig},
    timing::Duration,
};

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
}

impl SpringMotionDefaults {
    /// Creates spring motion defaults.
    #[must_use]
    pub const fn new(response: Duration, damping_ratio: f32) -> Self {
        Self {
            response,
            damping_ratio,
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

    /// Builds stable spring sampling configuration from these defaults.
    #[must_use]
    pub fn config(self) -> SpringConfig {
        self.into()
    }

    /// Builds a scalar spring from these defaults.
    #[must_use]
    pub fn scalar(self, from: f32, to: f32) -> ScalarSpring {
        ScalarSpring::new(from, to, self)
    }

    /// Builds a scalar spring with an initial velocity from these defaults.
    #[must_use]
    pub fn scalar_with_velocity(self, from: f32, to: f32, velocity: f32) -> ScalarSpring {
        self.scalar(from, to).with_initial_velocity(velocity)
    }
}

impl Default for SpringMotionDefaults {
    fn default() -> Self {
        Self::new(Duration::from_millis(280.0), 0.82)
    }
}
