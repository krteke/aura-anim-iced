//! Timing configuration and elapsed-time normalization.

mod duration;
mod iteration;
mod mode;
mod utils;

pub use duration::{Delay, Duration};
pub use iteration::IterationCount;
pub use mode::Direction;

pub use lilt::Easing;

/// Timing configuration shared by duration-based animations.
///
/// # Examples
///
/// ```
/// use aura_anim_core::timing::{Delay, Direction, Timing};
///
/// let timing = Timing::ease_out(250.0)
///     .with_delay(Delay::from_millis(50.0))
///     .with_direction(Direction::Alternate)
///     .with_iterations(2);
///
/// assert_eq!(timing.duration().as_millis(), 250.0);
/// assert_eq!(timing.total_duration().unwrap().as_millis(), 550.0);
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
}

impl Timing {
    /// Creates a timing value with a duration in milliseconds.
    #[must_use]
    pub fn new(duration_ms: impl Into<Duration>) -> Self {
        Self {
            duration: duration_ms.into(),
            ..Self::default()
        }
    }

    /// Creates a linear timing with a duration in milliseconds.
    #[must_use]
    pub fn linear(duration_ms: impl Into<Duration>) -> Self {
        Self::new(duration_ms)
    }

    /// Creates an ease-in timing with a duration in milliseconds.
    #[must_use]
    pub fn ease_in(duration_ms: impl Into<Duration>) -> Self {
        Self::new(duration_ms).with_easing(Easing::EaseIn)
    }

    /// Creates an ease-out timing with a duration in milliseconds.
    #[must_use]
    pub fn ease_out(duration_ms: impl Into<Duration>) -> Self {
        Self::new(duration_ms).with_easing(Easing::EaseOut)
    }

    /// Creates an ease-in-out timing with a duration in milliseconds.
    #[must_use]
    pub fn ease_in_out(duration_ms: impl Into<Duration>) -> Self {
        Self::new(duration_ms).with_easing(Easing::EaseInOut)
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

    /// Sets the duration.
    #[must_use]
    pub const fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
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

    pub(crate) fn with_rate(mut self, rate: f64) -> Self {
        self.duration = self.duration.divided_by(rate);
        self
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Delay, Timing};
    use float_cmp::assert_approx_eq;

    #[test]
    fn rate_scales_active_duration_without_changing_delay() {
        let faster = Timing::new(200.0)
            .with_delay(Delay::from_millis(40.0))
            .with_iterations(3)
            .with_rate(2.0);
        let slower = Timing::new(200.0).with_rate(0.5);

        assert_approx_eq!(f64, faster.duration().as_millis(), 100.0);
        assert_approx_eq!(f64, faster.delay().as_millis(), 40.0);
        assert_approx_eq!(f64, faster.total_duration().unwrap().as_millis(), 340.0);
        assert_approx_eq!(f64, slower.duration().as_millis(), 400.0);
    }

    #[test]
    fn invalid_rate_leaves_duration_unchanged() {
        let timing = Timing::new(200.0);

        assert_eq!(timing.with_rate(0.0).duration(), timing.duration());
        assert_eq!(timing.with_rate(-1.0).duration(), timing.duration());
        assert_eq!(timing.with_rate(f64::NAN).duration(), timing.duration());
        assert_eq!(
            timing.with_rate(f64::INFINITY).duration(),
            timing.duration()
        );
    }
}
