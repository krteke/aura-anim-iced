//! Timing configuration and elapsed-time normalization.

use std::{num::NonZeroU32, time::Duration as StdDuration};

pub use iced::animation::Easing;

#[cfg(test)]
mod tests;

/// A non-negative animation duration.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Duration(StdDuration);

impl Duration {
    /// A zero-length duration.
    pub const ZERO: Self = Self(StdDuration::ZERO);

    /// Creates a duration from milliseconds.
    #[must_use]
    pub fn from_millis(millis: f64) -> Self {
        let secs = sanitize_non_negative(millis) / 1000.0;
        Self(StdDuration::from_secs_f64(secs))
    }

    /// Creates a duration from seconds.
    #[must_use]
    pub fn from_secs(seconds: f64) -> Self {
        let secs = sanitize_non_negative(seconds);
        Self(StdDuration::from_secs_f64(secs))
    }

    /// Returns this duration in milliseconds.
    #[must_use]
    pub const fn as_millis(self) -> f64 {
        self.0.as_secs_f64() * 1000.0
    }
}

/// A non-negative animation start delay.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Delay(StdDuration);

impl Delay {
    /// No start delay.
    pub const ZERO: Self = Self(StdDuration::ZERO);

    /// Creates a delay from milliseconds.
    #[must_use]
    pub fn from_millis(millis: f64) -> Self {
        let secs = sanitize_non_negative(millis) / 1000.0;
        Self(StdDuration::from_secs_f64(secs))
    }

    /// Creates a delay from seconds.
    #[must_use]
    pub fn from_secs(seconds: f64) -> Self {
        let secs = sanitize_non_negative(seconds);
        Self(StdDuration::from_secs_f64(secs))
    }

    /// Returns this delay in milliseconds.
    #[must_use]
    pub const fn as_millis(self) -> f64 {
        self.0.as_secs_f64() * 1000.0
    }
}

/// Playback direction applied to repeated iterations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Play each iteration from start to end.
    #[default]
    Normal,
    /// Play each iteration from end to start.
    Reverse,
    /// Alternate forward and reverse iterations.
    Alternate,
    /// Alternate reverse and forward iterations.
    AlternateReverse,
}

/// How samples outside the active interval should be filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillMode {
    /// Do not fill before or after the active interval.
    #[default]
    None,
    /// Fill with the first active sample before the delay completes.
    Backwards,
    /// Fill with the final active sample after completion.
    Forwards,
    /// Fill both before the delay and after completion.
    Both,
}

/// Iteration configuration for a timing value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IterationCount {
    kind: IterationCountKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IterationCountKind {
    /// Repeat the animation a finite number of times.
    Count(NonZeroU32),
    /// Repeat the animation indefinitely.
    Infinite,
}

impl IterationCount {
    /// A single animation iteration.
    pub const ONCE: Self = Self {
        kind: IterationCountKind::Count(NonZeroU32::MIN),
    };

    /// An infinite number of iterations.
    pub const INFINITE: Self = Self {
        kind: IterationCountKind::Infinite,
    };

    /// Creates a finite iteration count, clamped to at least one iteration.
    #[must_use]
    pub fn count(count: u32) -> Self {
        let count = NonZeroU32::new(count).unwrap_or(NonZeroU32::MIN);

        Self {
            kind: IterationCountKind::Count(count),
        }
    }

    /// Returns an infinite iteration count.
    #[must_use]
    pub const fn infinite() -> Self {
        Self::INFINITE
    }

    /// Returns the finite count when this value is not infinite.
    #[must_use]
    pub const fn finite_count(self) -> Option<u32> {
        match self.kind {
            IterationCountKind::Count(count) => Some(count.get()),
            IterationCountKind::Infinite => None,
        }
    }
}

impl Default for IterationCount {
    fn default() -> Self {
        Self::ONCE
    }
}

impl From<u32> for IterationCount {
    fn from(value: u32) -> Self {
        Self::count(value)
    }
}

/// Timing state for an animation track or timeline step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Timing {
    /// Active duration for one iteration.
    pub duration: Duration,
    /// Start delay before the active interval.
    pub delay: Delay,
    /// Playback direction configuration.
    pub direction: Direction,
    /// Fill behavior outside the active interval.
    pub fill_mode: FillMode,
    /// Easing curve applied to normalized iteration progress.
    pub easing: Easing,
    /// Number of active iterations.
    pub iterations: IterationCount,
    /// Elapsed-time multiplier. Values at or below zero are normalized to `1.0`.
    pub playback_rate: f64,
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

        Some(Duration::from_millis(
            self.duration.as_millis() * f64::from(count),
        ))
    }

    /// Returns the total duration including delay when finite.
    #[must_use]
    pub fn total_duration(self) -> Option<Duration> {
        let active = self.active_duration()?;

        Some(Duration::from_millis(
            self.delay.as_millis() + active.as_millis(),
        ))
    }

    /// Normalizes elapsed milliseconds into active timing coordinates.
    #[must_use]
    pub fn normalize_elapsed(self, elapsed_ms: f64) -> NormalizedTiming {
        let elapsed_ms = sanitize_non_negative(elapsed_ms);
        let scaled_elapsed = elapsed_ms * self.playback_rate;
        let delay_ms = self.delay.as_millis();

        if scaled_elapsed < delay_ms {
            return NormalizedTiming::before_start();
        }

        let active_elapsed = scaled_elapsed - delay_ms;
        let duration_ms = self.duration.as_millis();

        if duration_ms == 0.0 {
            return NormalizedTiming::instant_complete();
        }

        let completed_iterations = completed_iterations_from(active_elapsed / duration_ms);

        if let Some(iteration_count) = self.iterations.finite_count()
            && completed_iterations >= iteration_count
        {
            return NormalizedTiming::after_end(iteration_count);
        }

        let iteration_elapsed = active_elapsed % duration_ms;
        let iteration_progress = iteration_elapsed / duration_ms;

        NormalizedTiming {
            phase: TimingPhase::Active,
            iteration_index: completed_iterations,
            iteration_progress,
            eased_iteration_progress: sample_easing(self.easing, iteration_progress),
            active_progress: active_elapsed / duration_ms,
        }
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

/// The broad phase produced by elapsed-time normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingPhase {
    /// Elapsed time is still before the active interval.
    BeforeStart,
    /// Elapsed time is inside the active interval.
    Active,
    /// Elapsed time is after a finite active interval.
    AfterEnd,
}

/// Normalized timing coordinates for sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedTiming {
    /// Current timing phase.
    pub phase: TimingPhase,
    /// Zero-based active iteration index.
    pub iteration_index: u32,
    /// Normalized progress inside the current iteration.
    pub iteration_progress: f64,
    /// Eased progress inside the current iteration.
    pub eased_iteration_progress: f64,
    /// Unclamped progress across active iterations.
    pub active_progress: f64,
}

impl NormalizedTiming {
    const fn before_start() -> Self {
        Self {
            phase: TimingPhase::BeforeStart,
            iteration_index: 0,
            iteration_progress: 0.0,
            eased_iteration_progress: 0.0,
            active_progress: 0.0,
        }
    }

    const fn instant_complete() -> Self {
        Self {
            phase: TimingPhase::AfterEnd,
            iteration_index: 1,
            iteration_progress: 1.0,
            eased_iteration_progress: 1.0,
            active_progress: 1.0,
        }
    }

    const fn after_end(iteration_count: u32) -> Self {
        Self {
            phase: TimingPhase::AfterEnd,
            iteration_index: iteration_count,
            iteration_progress: 1.0,
            eased_iteration_progress: 1.0,
            active_progress: iteration_count as f64,
        }
    }
}

fn clamp_progress(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn sample_easing(easing: Easing, progress: f64) -> f64 {
    f64::from(easing.value(clamp_progress(progress) as f32)).clamp(0.0, 1.0)
}

fn sanitize_non_negative(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

fn sanitize_playback_rate(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
fn completed_iterations_from(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }

    if value >= f64::from(u32::MAX) {
        return u32::MAX;
    }

    value.floor() as u32
}
