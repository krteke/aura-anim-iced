//! Timing configuration and elapsed-time normalization.

mod duration;
mod iteration;
mod mode;
mod normalized;
mod utils;

pub use duration::{Delay, Duration};
pub use iced::animation::Easing;
pub use iteration::IterationCount;
pub use mode::{Direction, FillMode};
pub use normalized::{NormalizedTiming, TimingPhase, TimingSampleState};

#[cfg(test)]
mod tests;

use crate::{
    nearly_equal_f64,
    timing::utils::{completed_iterations_from, sanitize_non_negative, sanitize_playback_rate},
};

/// Timing state for an animation track or timeline step.
///
/// # Example
///
/// ```
/// use aura_anim_iced::timing::{
///     Delay, Direction, Easing, FillMode, Timing, TimingSampleState,
/// };
///
/// let timing = Timing::new(200.0)
///     .with_delay(Delay::from_millis(50.0))
///     .with_direction(Direction::Alternate)
///     .with_fill_mode(FillMode::Both)
///     .with_easing(Easing::EaseOut)
///     .with_iterations(2);
///
/// let before = timing.normalize_elapsed(25.0);
/// let active = timing.normalize_elapsed(100.0);
///
/// assert_eq!(before.sample_state, TimingSampleState::BackwardsFill);
/// assert!(active.has_sample());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timing {
    /// Active duration for one iteration.
    duration: Duration,
    /// Start delay before the active interval.
    delay: Delay,
    /// Playback direction configuration.
    direction: Direction,
    /// Fill behavior outside the active interval.
    fill_mode: FillMode,
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

    /// Returns the fill mode of the timing.
    #[must_use]
    pub const fn fill_mode(&self) -> FillMode {
        self.fill_mode
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

    /// Sets the fill mode.
    #[must_use]
    pub const fn with_fill_mode(mut self, fill_mode: FillMode) -> Self {
        self.fill_mode = fill_mode;
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

    /// Normalizes elapsed milliseconds into active timing coordinates.
    #[must_use]
    pub fn normalize_elapsed(self, elapsed_ms: f64) -> NormalizedTiming {
        let elapsed_ms = sanitize_non_negative(elapsed_ms);
        let scaled_elapsed = elapsed_ms * self.playback_rate;
        let delay_ms = self.delay.as_millis();

        if scaled_elapsed < delay_ms {
            return NormalizedTiming::before_start(self.fill_mode, self.direction.start_progress());
        }

        let active_elapsed = scaled_elapsed - delay_ms;
        let duration_ms = self.duration.as_millis();

        if nearly_equal_f64(duration_ms, 0.0) {
            let count = self.iterations.finite_count().unwrap_or(1);

            return NormalizedTiming::after_end(
                count,
                self.fill_mode,
                self.direction.end_progress(count),
            );
        }

        let active_progress = active_elapsed / duration_ms;
        let completed_iterations = completed_iterations_from(active_progress);

        if let Some(iteration_count) = self.iterations.finite_count()
            && completed_iterations >= iteration_count
        {
            return NormalizedTiming::after_end(
                iteration_count,
                self.fill_mode,
                self.direction.end_progress(iteration_count),
            );
        }

        if !active_progress.is_finite() {
            let directed_iteration_progress =
                self.direction.sample_progress(completed_iterations, 0.0);

            return NormalizedTiming::active(
                completed_iterations,
                directed_iteration_progress,
                f64::from(completed_iterations),
            );
        }

        let iteration_elapsed = active_elapsed % duration_ms;
        let raw_iteration_progress = iteration_elapsed / duration_ms;
        let directed_iteration_progress = self
            .direction
            .sample_progress(completed_iterations, raw_iteration_progress);

        NormalizedTiming::active(
            completed_iterations,
            directed_iteration_progress,
            active_progress,
        )
    }
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            duration: Duration::ZERO,
            delay: Delay::ZERO,
            direction: Direction::default(),
            fill_mode: FillMode::default(),
            easing: Easing::Linear,
            iterations: IterationCount::default(),
            playback_rate: 1.0,
        }
    }
}
