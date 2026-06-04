//! Spring motion sampling primitives.

mod sample;
#[cfg(test)]
mod tests;

use crate::{nearly_equal_f64, prelude::SpringMotionDefaults, timing::Duration};
use sample::{sample_critical, sample_overdamped, sample_underdamped};

pub use sample::ScalarSpringSample;

/// Configuration for stable spring sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    response: Duration,
    damping_ratio: f32,
}

impl SpringConfig {
    /// Creates spring sampling configuration.
    ///
    /// `response` controls the spring period and `damping_ratio` controls how
    /// quickly oscillation decays. A damping ratio below `1.0` can overshoot,
    /// `1.0` is critically damped, and values above `1.0` are overdamped.
    #[must_use]
    pub fn new(response: Duration, damping_ratio: f32) -> Self {
        Self {
            response,
            damping_ratio: sanitize_damping_ratio(damping_ratio),
        }
    }

    /// Returns the response duration.
    #[must_use]
    pub const fn response(self) -> Duration {
        self.response
    }

    /// Returns the sanitized damping ratio.
    #[must_use]
    pub const fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }
}

impl From<SpringMotionDefaults> for SpringConfig {
    fn from(value: crate::defaults::SpringMotionDefaults) -> Self {
        Self::new(value.response(), value.damping_ratio())
    }
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self::new(Duration::from_millis(280.0), 0.82)
    }
}

/// A scalar spring from one visual value to another.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarSpring {
    from: f32,
    to: f32,
    initial_velocity: f32,
    config: SpringConfig,
}

impl ScalarSpring {
    /// Creates a scalar spring with zero initial velocity.
    #[must_use]
    pub fn new(from: f32, to: f32, config: impl Into<SpringConfig>) -> Self {
        Self {
            from: sanitize_scalar(from),
            to: sanitize_scalar(to),
            initial_velocity: 0.0,
            config: config.into(),
        }
    }

    /// Sets the initial velocity in value units per second.
    #[must_use]
    pub fn with_initial_velocity(mut self, velocity: f32) -> Self {
        self.initial_velocity = sanitize_scalar(velocity);
        self
    }

    /// Returns the starting value.
    #[must_use]
    pub const fn from(self) -> f32 {
        self.from
    }

    /// Returns the target value.
    #[must_use]
    pub const fn to(self) -> f32 {
        self.to
    }

    /// Returns the initial velocity in value units per second.
    #[must_use]
    pub const fn initial_velocity(self) -> f32 {
        self.initial_velocity
    }

    /// Returns the spring configuration.
    #[must_use]
    pub const fn config(self) -> SpringConfig {
        self.config
    }

    /// Samples the spring at `elapsed`.
    #[must_use]
    pub fn sample_at(self, elapsed: Duration) -> ScalarSpringSample {
        let response_secs = self.config.response().as_secs();

        if response_secs <= 0.0 {
            return ScalarSpringSample::new(self.to, 0.0);
        }

        let elapsed_secs = elapsed.as_secs();
        let omega = core::f64::consts::TAU / response_secs;
        let damping_ratio = f64::from(self.config.damping_ratio());
        let displacement = f64::from(self.from - self.to);
        let velocity = f64::from(self.initial_velocity);

        let sample = if damping_ratio < 1.0 {
            sample_underdamped(displacement, velocity, elapsed_secs, omega, damping_ratio)
        } else if nearly_equal_f64(damping_ratio, 1.0) {
            sample_critical(displacement, velocity, elapsed_secs, omega)
        } else {
            sample_overdamped(displacement, velocity, elapsed_secs, omega, damping_ratio)
        };

        ScalarSpringSample::new(
            sanitize_output(self.to, sample.displacement()),
            sanitize_output(0.0, sample.velocity()),
        )
    }
}

fn sanitize_scalar(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn sanitize_damping_ratio(value: f32) -> f32 {
    if value.is_finite() && value >= 0.0 {
        value
    } else {
        1.0
    }
}

#[allow(clippy::cast_possible_truncation)]
fn sanitize_output(target: f32, offset: f64) -> f32 {
    if offset.is_finite() {
        target + offset as f32
    } else {
        target
    }
}
