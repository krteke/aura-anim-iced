use crate::{interpolate::InterpolationProgress, timing::Duration};

pub trait Interpolate: Sized {
    fn lerp(&self, other: &Self, progress: f32) -> Self {
        Self::interpolate_progress(self, other, InterpolationProgress::new(progress))
    }

    fn interpolate(from: &Self, to: &Self, progress: f32) -> Self {
        Self::interpolate_progress(from, to, InterpolationProgress::new(progress))
    }

    fn extrapolate(from: &Self, to: &Self, progress: f32) -> Self {
        Self::interpolate_progress(from, to, InterpolationProgress::extrapolated(progress))
    }

    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self;
}

pub trait Animatable: Interpolate + Clone + 'static {}

impl<T: Interpolate + Clone + 'static> Animatable for T {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    Running,
    Paused,
    Completed,
    Canceled,
}

pub trait Animation<T: Animatable>: 'static {
    fn value(&self) -> &T;

    fn state(&self) -> AnimationState;

    fn duration(&self) -> Option<Duration> {
        None
    }

    fn tick(&mut self, delta: Duration);

    fn advance(&mut self, delta: Duration) -> Duration {
        self.tick(delta);
        Duration::ZERO
    }

    fn pause(&mut self);

    fn resume(&mut self);

    fn cancel(&mut self);

    fn seek(&mut self, progress: f32);

    fn finish(&mut self);

    fn retarget(&mut self, _target: &T) -> bool {
        false
    }

    fn is_active(&self) -> bool {
        self.state() == AnimationState::Running
    }
}
