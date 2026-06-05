//! Timing configuration and elapsed-time normalization.
use lilt::Easing;

mod duration;
mod iteration;
mod mode;
mod utils;

pub use duration::{Delay, Duration};
pub use iteration::IterationCount;
pub use mode::Direction;

use crate::timing::utils::sanitize_playback_rate;

/// Timing state for an animation track or timeline step.
///
/// # Example
///
/// ```
/// use aura_anim_iced::timing::{
///     Delay, Direction, Easing, Timing, TimingSampleState,
/// };
///
/// let timing = Timing::new(200.0)
///     .with_delay(Delay::from_millis(50.0))
///     .with_direction(Direction::Alternate)
///     .with_easing(Easing::EaseOut)
///     .with_iterations(2);
///
/// let before = timing.normalize_elapsed(25.0);
/// let active = timing.normalize_elapsed(100.0);
///
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timing {
    /// Active duration for one iteration.
    duration: Duration,
    /// Start delay before the active interval.
    delay: Delay,
    /// Playback direction configuration.
    direction: Direction,
    /// Easing curve applied to normalized iteration progress.
    easing: Easing,
    /// Number of active iterations.
    iterations: IterationCount,
    /// Elapsed-time multiplier. Values at or below zero are normalized to `1.0`.
    playback_rate: f64,
}

impl Timing {
    /// Creates a timing value with a duration in milliseconds.
    #[must_use]
    pub fn new(duration_ms: f64) -> Self {
        Self {
            duration: Duration::from_millis(duration_ms),
            ..Self::default()
        }
    }

    /// Returns the duration of the timing.
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the delay of the timing.
    #[must_use]
    pub const fn delay(&self) -> Delay {
        self.delay
    }

    /// Returns the direction of the timing.
    #[must_use]
    pub const fn direction(&self) -> Direction {
        self.direction
    }

    /// Returns the easing curve of the timing.
    #[must_use]
    pub const fn easing(&self) -> Easing {
        self.easing
    }

    /// Returns the number of iterations of the timing.
    #[must_use]
    pub const fn iterations(&self) -> IterationCount {
        self.iterations
    }

    /// Returns the playback rate of the timing.
    #[must_use]
    pub const fn playback_rate(&self) -> f64 {
        self.playback_rate
    }

    /// Sets the start delay.
    #[must_use]
    pub const fn with_delay(mut self, delay: Delay) -> Self {
        self.delay = delay;
        self
    }

    /// Sets the playback direction.
    #[must_use]
    pub const fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    /// Sets the easing curve.
    #[must_use]
    pub const fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the iteration count.
    #[must_use]
    pub fn with_iterations(mut self, iterations: impl Into<IterationCount>) -> Self {
        self.iterations = iterations.into();
        self
    }

    /// Sets the playback rate.
    #[must_use]
    pub fn with_playback_rate(mut self, playback_rate: f64) -> Self {
        self.playback_rate = sanitize_playback_rate(playback_rate);
        self
    }

    /// Returns the total active duration when the timing has a finite length.
    #[must_use]
    pub fn active_duration(self) -> Option<Duration> {
        let count = self.iterations.finite_count()?;

        self.duration.checked_mul(count)
    }

    /// Returns the total duration including delay when finite.
    #[must_use]
    pub fn total_duration(self) -> Option<Duration> {
        let active = self.active_duration()?;

        active.checked_add_delay(self.delay)
    }
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            duration: Duration::ZERO,
            delay: Delay::ZERO,
            direction: Direction::default(),
            easing: Easing::Linear,
            iterations: IterationCount::default(),
            playback_rate: 1.0,
        }
    }
}
