use crate::{
    timing::{Duration, Timing},
    traits::{Animatable, Animation, AnimationState},
};

pub type TweenState = AnimationState;

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
    pub fn new(value: T) -> Self {
        Self::with_timing(value, Timing::new(200.0))
    }

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

    pub fn between(from: T, to: T, timing: Timing) -> Self {
        let mut tween = Self::with_timing(from, timing);
        tween.transition_to(to);
        tween
    }

    pub fn value(&self) -> &T {
        &self.current
    }

    pub fn from(&self) -> &T {
        &self.from
    }

    pub fn target(&self) -> &T {
        &self.to
    }

    pub const fn timing(&self) -> Timing {
        self.timing
    }

    pub const fn state(&self) -> AnimationState {
        self.state
    }

    pub fn is_active(&self) -> bool {
        self.state == AnimationState::Running
    }

    pub fn is_completed(&self) -> bool {
        self.state == AnimationState::Completed
    }

    pub fn transition_to(&mut self, target: T) {
        self.from = self.current.clone();
        self.to = target;
        self.elapsed = Duration::ZERO;
        self.state = AnimationState::Running;
        self.sample();
    }

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

    pub fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Running;
        }
    }

    pub fn cancel(&mut self) {
        if matches!(
            self.state,
            AnimationState::Running | AnimationState::Paused | AnimationState::Idle
        ) {
            self.state = AnimationState::Canceled;
        }
    }

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
}
