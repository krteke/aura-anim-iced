use crate::{Animatable, Animation, AnimationState, timeline::normalized, timing::Duration};

/// An animation that keeps a value unchanged for a fixed duration.
#[derive(Debug, Clone)]
pub struct Hold<T: Animatable> {
    value: T,
    elapsed: Duration,
    duration: Duration,
    state: AnimationState,
}

impl<T: Animatable> Hold<T> {
    /// Creates a running hold animation.
    #[must_use]
    pub fn new(value: T, duration: impl Into<Duration>) -> Self {
        Self {
            value,
            elapsed: Duration::ZERO,
            duration: duration.into(),
            state: AnimationState::Running,
        }
    }
}

impl<T: Animatable> Animation<T> for Hold<T> {
    fn value(&self) -> &T {
        &self.value
    }

    fn state(&self) -> AnimationState {
        self.state
    }

    fn duration(&self) -> Option<Duration> {
        Some(self.duration)
    }

    fn tick(&mut self, delta: Duration) {
        self.advance(delta);
    }

    fn advance(&mut self, delta: Duration) -> Duration {
        if self.state != AnimationState::Running {
            return delta;
        }
        let remaining = self.duration.saturating_sub(self.elapsed);
        let consumed = delta.min(remaining);
        self.elapsed += consumed;
        if self.elapsed >= self.duration {
            self.state = AnimationState::Completed;
        }
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
        self.elapsed =
            Duration::from_secs(self.duration.as_secs() * f64::from(normalized(progress)));
        self.state = if normalized(progress) >= 1.0 {
            AnimationState::Completed
        } else {
            AnimationState::Running
        };
    }

    fn finish(&mut self) {
        self.elapsed = self.duration;
        self.state = AnimationState::Completed;
    }

    fn set_rate(&mut self, rate: f64) {
        self.duration = self.duration.divided_by(rate);
    }

    fn into_value(self: Box<Self>) -> T {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::Hold;
    use crate::{Animation, AnimationState, timing::Duration};

    #[test]
    fn advance_returns_overflow_after_completion() {
        let mut hold = Hold::new(4_i32, Duration::from_millis(50.0));

        let overflow = hold.advance(Duration::from_millis(80.0));

        assert_eq!(overflow, Duration::from_millis(30.0));
        assert_eq!(hold.state(), AnimationState::Completed);
        assert_eq!(*hold.value(), 4);
    }

    #[test]
    fn pause_resume_cancel_and_finish_update_state() {
        let mut hold = Hold::new(4_i32, Duration::from_millis(50.0));

        hold.pause();
        assert_eq!(hold.state(), AnimationState::Paused);
        hold.resume();
        assert_eq!(hold.state(), AnimationState::Running);
        hold.cancel();
        assert_eq!(hold.state(), AnimationState::Canceled);
        hold.finish();
        assert_eq!(hold.state(), AnimationState::Completed);
    }

    #[test]
    fn seek_clamps_progress_and_updates_state() {
        let mut hold = Hold::new(4_i32, Duration::from_millis(50.0));

        hold.seek(f32::NAN);
        assert_eq!(hold.state(), AnimationState::Running);
        hold.seek(2.0);
        assert_eq!(hold.state(), AnimationState::Completed);
    }

    #[test]
    fn rate_scales_hold_duration() {
        let hold = Hold::new(4_i32, Duration::from_millis(100.0)).rate(2.0);

        assert_eq!(hold.duration(), Some(Duration::from_millis(50.0)));
    }
}
