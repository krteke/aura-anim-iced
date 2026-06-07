use crate::{Animatable, Animation, AnimationState, timing::Duration};

pub(super) struct Settled<T: Animatable> {
    value: T,
    state: AnimationState,
}

impl<T: Animatable> Settled<T> {
    pub(super) fn new(value: T, state: AnimationState) -> Self {
        Self { value, state }
    }
}

impl<T: Animatable> Animation<T> for Settled<T> {
    fn value(&self) -> &T {
        &self.value
    }

    fn state(&self) -> AnimationState {
        self.state
    }

    fn duration(&self) -> Option<Duration> {
        Some(Duration::ZERO)
    }

    fn tick(&mut self, _delta: Duration) {}

    fn pause(&mut self) {}

    fn resume(&mut self) {}

    fn cancel(&mut self) {}

    fn seek(&mut self, _progress: f32) {}

    fn finish(&mut self) {
        self.state = AnimationState::Completed;
    }
}
