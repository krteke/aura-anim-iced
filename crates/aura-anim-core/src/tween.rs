//! Duration-based interpolation animations.

use crate::{
    timing::{Duration, Timing},
    traits::{Animatable, Animation, AnimationState},
};

/// The lifecycle state of a [`Tween`].
pub type TweenState = AnimationState;

/// An animation that interpolates between two values using [`Timing`].
///
/// # Examples
///
/// ```
/// use aura_anim_core::{Tween, timing::Timing};
/// use std::time::Duration;
///
/// let mut tween = Tween::between(0.0_f32, 10.0, Timing::new(100.0));
///
/// tween.tick(Duration::from_millis(50));
/// assert_eq!(*tween.value(), 5.0);
///
/// tween.tick(Duration::from_millis(50));
/// assert!(tween.is_completed());
/// assert_eq!(*tween.value(), 10.0);
/// ```
#[derive(Debug, Clone)]
pub struct Tween<T: Animatable> {
    from: T,
    to: T,
    current: T,
    elapsed: Duration,
    timing: Timing,
    state: AnimationState,
}

impl<T: Animatable> Tween<T> {
    /// Creates an idle tween with the default 200 millisecond timing.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self::with_timing(value, Timing::new(200.0))
    }

    /// Creates an idle tween with the provided timing.
    #[must_use]
    pub fn with_timing(value: T, timing: Timing) -> Self {
        Self {
            from: value.clone(),
            to: value.clone(),
            current: value,
            elapsed: Duration::ZERO,
            timing,
            state: AnimationState::Idle,
        }
    }

    /// Creates a running tween from `from` to `to`.
    #[must_use]
    pub fn between(from: T, to: T, timing: Timing) -> Self {
        let mut tween = Self::with_timing(from, timing);
        tween.transition_to(to);
        tween
    }

    /// Returns the current interpolated value.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.current
    }

    /// Returns the value at the start of the current transition.
    #[must_use]
    pub fn from(&self) -> &T {
        &self.from
    }

    /// Returns the target value of the current transition.
    #[must_use]
    pub fn target(&self) -> &T {
        &self.to
    }

    /// Returns the timing configuration.
    #[must_use]
    pub const fn timing(&self) -> Timing {
        self.timing
    }

    /// Returns the current lifecycle state.
    #[must_use]
    pub const fn state(&self) -> AnimationState {
        self.state
    }

    /// Returns whether the tween is currently running.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state == AnimationState::Running
    }

    /// Returns whether the tween has completed.
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.state == AnimationState::Completed
    }

    /// Starts a transition from the current value to `target`.
    pub fn transition_to(&mut self, target: T) {
        self.from = self.current.clone();
        self.to = target;
        self.elapsed = Duration::ZERO;
        self.state = AnimationState::Running;
        self.sample();
    }

    /// Advances the tween by `delta`.
    pub fn tick(&mut self, delta: impl Into<Duration>) {
        if self.state != AnimationState::Running {
            return;
        }

        self.elapsed += delta.into();
        self.sample();
    }

    fn remaining(&self) -> Option<Duration> {
        let total = self.timing.total_duration()?;
        Some(total.saturating_sub(self.elapsed))
    }

    /// Pauses the tween when it is running.
    pub fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
        }
    }

    /// Resumes the tween when it is paused.
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Running;
        }
    }

    /// Cancels the tween unless it is already completed or canceled.
    pub fn cancel(&mut self) {
        if matches!(
            self.state,
            AnimationState::Running | AnimationState::Paused | AnimationState::Idle
        ) {
            self.state = AnimationState::Canceled;
        }
    }

    /// Seeks to normalized progress within the complete timing interval.
    pub fn seek(&mut self, progress: f32) {
        let progress = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        self.state = AnimationState::Running;
        let total = self.timing.total_duration().unwrap_or_else(|| {
            Duration::from_millis(
                self.timing.delay().as_millis() + self.timing.duration().as_millis(),
            )
        });
        self.elapsed = Duration::from_millis(total.as_millis() * f64::from(progress));
        self.sample();
    }

    /// Moves the tween to its final value and completed state.
    #[allow(clippy::cast_possible_truncation)]
    pub fn finish(&mut self) {
        let progress = self
            .timing
            .iterations()
            .finite_count()
            .map_or(self.timing.direction().sample_progress(1, 1.0), |count| {
                self.timing.direction().end_progress(count)
            });
        self.current = T::interpolate(
            &self.from,
            &self.to,
            self.timing.easing().value(progress as f32),
        );
        self.state = AnimationState::Completed;
    }

    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_possible_truncation)]
    fn sample(&mut self) {
        let delay = self.timing.delay().as_millis();
        let elapsed = self.elapsed.as_millis();

        if elapsed < delay {
            self.current = self.from.clone();
            return;
        }

        let duration = self.timing.duration().as_millis();
        if duration <= 0.0 {
            self.finish();
            return;
        }

        let active_elapsed = elapsed - delay;
        let iterations = self.timing.iterations().finite_count();

        if let Some(count) = iterations {
            let total = duration * f64::from(count);
            if active_elapsed >= total {
                self.finish();
                return;
            }
        }

        let iteration = (active_elapsed / duration).floor() as u32;
        let raw_progress = (active_elapsed % duration) / duration;
        let progress = self
            .timing
            .direction()
            .sample_progress(iteration, raw_progress);
        let eased = self.timing.easing().value(progress as f32);
        self.current = T::interpolate(&self.from, &self.to, eased);
    }
}

impl<T: Animatable> Animation<T> for Tween<T> {
    fn value(&self) -> &T {
        self.value()
    }

    fn state(&self) -> AnimationState {
        self.state()
    }

    fn duration(&self) -> Option<Duration> {
        self.timing.total_duration()
    }

    fn tick(&mut self, delta: Duration) {
        self.tick(delta);
    }

    fn advance(&mut self, delta: Duration) -> Duration {
        if self.state != AnimationState::Running {
            return delta;
        }

        let Some(remaining) = self.remaining() else {
            self.tick(delta);
            return Duration::ZERO;
        };
        let consumed = delta.min(remaining);
        self.tick(consumed);
        delta.saturating_sub(consumed)
    }

    fn pause(&mut self) {
        self.pause();
    }

    fn resume(&mut self) {
        self.resume();
    }

    fn cancel(&mut self) {
        self.cancel();
    }

    fn seek(&mut self, progress: f32) {
        self.seek(progress);
    }

    fn finish(&mut self) {
        self.finish();
    }

    fn retarget(&mut self, target: &T) -> bool {
        self.transition_to(target.clone());
        true
    }

    fn into_value(self: Box<Self>) -> T {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::Tween;
    use crate::{
        Animation, AnimationState,
        timing::{Direction, Duration, IterationCount, Timing},
    };
    use float_cmp::assert_approx_eq;

    #[test]
    fn new_tween_is_idle_with_matching_endpoints() {
        let tween = Tween::new(3.0_f32);

        assert_eq!(tween.state(), AnimationState::Idle);
        assert_approx_eq!(f32, *tween.value(), 3.0);
        assert_approx_eq!(f32, *tween.from(), 3.0);
        assert_approx_eq!(f32, *tween.target(), 3.0);
        assert_approx_eq!(f64, tween.timing().duration().as_millis(), 200.0);
    }

    #[test]
    fn transition_uses_current_value_as_new_start() {
        let mut tween = Tween::between(0.0_f32, 10.0, Timing::new(100.0));
        tween.tick(Duration::from_millis(40.0));
        tween.transition_to(20.0);

        assert_approx_eq!(f32, *tween.from(), 4.0);
        assert_approx_eq!(f32, *tween.value(), 4.0);
        assert_approx_eq!(f32, *tween.target(), 20.0);
    }

    #[test]
    fn paused_and_canceled_tweens_do_not_tick() {
        let mut tween = Tween::between(0.0_f32, 10.0, Timing::new(100.0));
        tween.pause();
        tween.tick(Duration::from_millis(50.0));
        assert_approx_eq!(f32, *tween.value(), 0.0);

        tween.resume();
        tween.tick(Duration::from_millis(25.0));
        assert_approx_eq!(f32, *tween.value(), 2.5);

        tween.cancel();
        tween.tick(Duration::from_millis(75.0));
        assert_eq!(tween.state(), AnimationState::Canceled);
        assert_approx_eq!(f32, *tween.value(), 2.5);
    }

    #[test]
    fn advance_returns_unconsumed_duration() {
        let mut tween = Tween::between(0.0_f32, 1.0, Timing::new(100.0));

        let overflow = Animation::advance(&mut tween, Duration::from_millis(125.0));

        assert_eq!(overflow, Duration::from_millis(25.0));
        assert_eq!(tween.state(), AnimationState::Completed);
    }

    #[test]
    fn infinite_tween_consumes_all_advanced_time() {
        let timing = Timing::new(100.0).with_iterations(IterationCount::INFINITE);
        let mut tween = Tween::between(0.0_f32, 1.0, timing);

        let overflow = Animation::advance(&mut tween, Duration::from_millis(250.0));

        assert_eq!(overflow, Duration::ZERO);
        assert_eq!(tween.state(), AnimationState::Running);
        assert_approx_eq!(f32, *tween.value(), 0.5);
    }

    #[test]
    fn finish_respects_repeated_direction() {
        let timing = Timing::new(100.0)
            .with_iterations(2)
            .with_direction(Direction::Alternate);
        let mut tween = Tween::between(0.0_f32, 10.0, timing);

        tween.finish();

        assert_eq!(tween.state(), AnimationState::Completed);
        assert_approx_eq!(f32, *tween.value(), 0.0);
    }

    #[test]
    fn zero_duration_tween_finishes_on_tick() {
        let mut tween = Tween::between(0.0_f32, 10.0, Timing::new(0.0));

        tween.tick(Duration::ZERO);

        assert_eq!(tween.state(), AnimationState::Completed);
        assert_approx_eq!(f32, *tween.value(), 10.0);
    }
}
