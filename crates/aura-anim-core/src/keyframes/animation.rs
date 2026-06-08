use std::cmp::Ordering;

use lilt::Easing;

use crate::{
    keyframes::Keyframe,
    timing::{Delay, Direction, Duration, IterationCount, Timing},
    traits::{Animatable, Animation, AnimationState},
};

/// An animation composed of time-positioned [`Keyframe`] values.
///
/// # Examples
///
/// ```
/// use aura_anim_core::{Animation, keyframes::Keyframes, timing::Duration};
///
/// let mut animation = Keyframes::new(0.0_f32)
///     .push(100.0, 1.0)
///     .push(200.0, 0.0);
///
/// animation.tick(Duration::from_millis(100.0));
/// assert_eq!(*animation.value(), 1.0);
///
/// animation.tick(Duration::from_millis(100.0));
/// assert_eq!(*animation.value(), 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct Keyframes<T: Animatable> {
    frames: Vec<Keyframe<T>>,
    current: T,
    elapsed: Duration,
    timing: Timing,
    state: AnimationState,
}

impl<T: Animatable> Keyframes<T> {
    /// Creates a running keyframe animation with an initial frame at zero.
    #[must_use]
    pub fn new(initial: T) -> Self {
        Self {
            frames: vec![Keyframe::new(0.0, initial.clone())],
            current: initial,
            elapsed: Duration::ZERO,
            timing: Timing::default(),
            state: AnimationState::Running,
        }
    }

    /// Adds a keyframe with linear easing after the preceding frame.
    #[must_use]
    pub fn push(self, time_ms: f64, value: T) -> Self {
        self.push_eased(time_ms, value, Easing::Linear)
    }

    /// Adds a keyframe with the provided easing.
    #[must_use]
    pub fn push_eased(self, time_ms: f64, value: T, easing: Easing) -> Self {
        let frame = Keyframe::new(time_ms.max(0.0), value).with_easing(easing);
        self.push_frame(frame)
    }

    /// Adds or replaces a keyframe at the frame's time.
    #[must_use]
    pub fn push_frame(mut self, frame: Keyframe<T>) -> Self {
        match self.frames.binary_search_by(|existing| {
            existing
                .time()
                .partial_cmp(&frame.time())
                .unwrap_or(Ordering::Equal)
        }) {
            Ok(index) => self.frames[index] = frame,
            Err(index) => self.frames.insert(index, frame),
        }
        self.timing = self.timing.with_duration(self.duration());
        self
    }

    /// Sets the delay before playback begins.
    #[must_use]
    pub fn with_delay(mut self, delay: Delay) -> Self {
        self.timing = self.timing.with_delay(delay);
        self
    }

    /// Sets the number of playback iterations.
    #[must_use]
    pub fn with_iterations(mut self, iterations: impl Into<IterationCount>) -> Self {
        self.timing = self.timing.with_iterations(iterations);
        self
    }

    /// Sets the playback direction.
    #[must_use]
    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.timing = self.timing.with_direction(direction);
        self
    }

    /// Returns the time of the final keyframe.
    #[must_use]
    pub fn duration(&self) -> Duration {
        Duration::from_millis(self.frames.last().map_or(0.0, Keyframe::time))
    }

    fn sample(&mut self) {
        let delay = self.timing.delay().as_millis();
        let elapsed = self.elapsed.as_millis();
        if elapsed < delay {
            self.current = self.frames[0].value().clone();
            return;
        }

        let duration = self.duration().as_millis();
        if duration <= 0.0 {
            self.finish();
            return;
        }

        let active_elapsed = elapsed - delay;
        if let Some(count) = self.timing.iterations().finite_count() {
            let total = duration * f64::from(count);
            if active_elapsed >= total {
                self.finish();
                return;
            }
        }

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let iteration = (active_elapsed / duration).floor() as u32;

        let raw_progress = (active_elapsed % duration) / duration;
        let progress = self
            .timing
            .direction()
            .sample_progress(iteration, raw_progress);
        self.current = self.value_at(duration * progress);
    }

    fn value_at(&self, time_ms: f64) -> T {
        if time_ms <= self.frames[0].time() {
            return self.frames[0].value().clone();
        }

        let upper = self.frames.partition_point(|frame| frame.time() <= time_ms);
        if upper >= self.frames.len() {
            return self.frames[self.frames.len() - 1].value().clone();
        }

        let from = &self.frames[upper - 1];
        let to = &self.frames[upper];
        let span = (to.time() - from.time()).max(f64::EPSILON);
        let progress = ((time_ms - from.time()) / span).clamp(0.0, 1.0);

        #[allow(clippy::cast_possible_truncation)]
        let eased = from.easing().value(progress as f32);

        T::interpolate(from.value(), to.value(), eased)
    }
}

impl<T: Animatable> Animation<T> for Keyframes<T> {
    fn value(&self) -> &T {
        &self.current
    }

    fn state(&self) -> AnimationState {
        self.state
    }

    fn duration(&self) -> Option<Duration> {
        self.timing.total_duration()
    }

    fn tick(&mut self, delta: Duration) {
        if self.state != AnimationState::Running {
            return;
        }
        self.elapsed += delta;
        self.sample();
    }

    fn advance(&mut self, delta: Duration) -> Duration {
        if self.state != AnimationState::Running {
            return delta;
        }

        let Some(total) = self.timing.total_duration() else {
            self.tick(delta);
            return Duration::ZERO;
        };
        let remaining = total.saturating_sub(self.elapsed);
        let consumed = delta.min(remaining);
        self.tick(consumed);
        delta.saturating_sub(consumed)
    }

    fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
        }
    }

    fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Running;
        }
    }

    fn cancel(&mut self) {
        if matches!(self.state, AnimationState::Running | AnimationState::Paused) {
            self.state = AnimationState::Canceled;
        }
    }

    fn seek(&mut self, progress: f32) {
        let progress = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        self.state = AnimationState::Running;
        let total = self
            .timing
            .total_duration()
            .unwrap_or(self.duration())
            .as_millis();
        self.elapsed = Duration::from_millis(total * f64::from(progress));
        self.sample();
    }

    fn finish(&mut self) {
        let progress = self
            .timing
            .iterations()
            .finite_count()
            .map_or(self.timing.direction().sample_progress(1, 1.0), |count| {
                self.timing.direction().end_progress(count)
            });
        self.current = self.value_at(self.duration().as_millis() * progress);
        self.state = AnimationState::Completed;
    }
}
