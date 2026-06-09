//! Core traits implemented by animatable values and animation sources.

use crate::{interpolate::InterpolationProgress, timing::Duration};

/// Interpolates values of the same type.
pub trait Interpolate: Sized {
    /// Interpolates from `self` to `other` using normalized progress.
    #[must_use]
    fn lerp(&self, other: &Self, progress: f32) -> Self {
        Self::interpolate_progress(self, other, InterpolationProgress::new(progress))
    }

    /// Interpolates from `from` to `to` using normalized progress.
    #[must_use]
    fn interpolate(from: &Self, to: &Self, progress: f32) -> Self {
        Self::interpolate_progress(from, to, InterpolationProgress::new(progress))
    }

    /// Interpolates or extrapolates from `from` to `to` without clamping finite progress.
    #[must_use]
    fn extrapolate(from: &Self, to: &Self, progress: f32) -> Self {
        Self::interpolate_progress(from, to, InterpolationProgress::extrapolated(progress))
    }

    /// Interpolates from `from` to `to` using a prepared progress value.
    #[must_use]
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self;
}

/// Marker trait for cloneable, owned values that can be animated.
pub trait Animatable: Interpolate + Clone + 'static {}

impl<T: Interpolate + Clone + 'static> Animatable for T {}

/// Lifecycle state of an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    /// The animation has not started.
    Idle,
    /// The animation is advancing.
    Running,
    /// The animation is suspended and retains its current value.
    Paused,
    /// The animation reached its final value.
    Completed,
    /// The animation was canceled before completion.
    Canceled,
}

/// A stateful source that produces animated values over time.
pub trait Animation<T: Animatable>: 'static {
    /// Returns the animation's current value.
    #[must_use]
    fn value(&self) -> &T;

    /// Returns the animation's lifecycle state.
    #[must_use]
    fn state(&self) -> AnimationState;

    /// Returns the total duration when it is finite and known.
    #[must_use]
    fn duration(&self) -> Option<Duration> {
        None
    }

    /// Advances the animation by `delta`.
    fn tick(&mut self, delta: Duration);

    /// Advances the animation and returns any unconsumed duration.
    fn advance(&mut self, delta: Duration) -> Duration {
        self.tick(delta);
        Duration::ZERO
    }

    /// Pauses a running animation.
    fn pause(&mut self);

    /// Resumes a paused animation.
    fn resume(&mut self);

    /// Cancels the animation.
    fn cancel(&mut self);

    /// Seeks to normalized progress within the animation.
    fn seek(&mut self, progress: f32);

    /// Moves the animation to its completed state.
    fn finish(&mut self);

    /// Attempts to continue the animation toward a new target.
    ///
    /// Returns `true` when the animation supports retargeting.
    #[must_use]
    fn retarget(&mut self, _target: &T) -> bool {
        false
    }

    /// Returns whether the animation is currently running.
    #[must_use]
    fn is_active(&self) -> bool {
        self.state() == AnimationState::Running
    }

    /// Updates playback rate by adjusting the animation's stored durations.
    ///
    /// The default implementation does nothing, which is appropriate for
    /// animations such as springs that are not duration-based.
    fn set_rate(&mut self, _rate: f64) {}

    /// Returns this animation with its playback rate adjusted.
    ///
    /// A rate of `2.0` halves duration, while `0.5` doubles it. Repeated calls
    /// compound because implementations update duration directly instead of
    /// storing a separate rate value.
    #[must_use]
    fn rate(mut self, rate: f64) -> Self
    where
        Self: Sized,
    {
        self.set_rate(rate);
        self
    }

    /// Consumes the animation and returns its current value.
    ///
    /// The default implementation clones the sampled value so custom
    /// animations remain source-compatible. Implementations that own their
    /// sampled value should override this method to move it without cloning.
    #[must_use]
    fn into_value(self: Box<Self>) -> T {
        self.value().clone()
    }
}

/// A type-erased animation source.
pub type BoxAnimation<T> = Box<dyn Animation<T>>;

/// Convenience methods for concrete animation sources.
pub trait AnimationExt<T: Animatable>: Animation<T> + Sized {
    /// Type-erases this animation for use in transition factories.
    #[must_use]
    fn boxed(self) -> BoxAnimation<T> {
        Box::new(self)
    }
}

impl<T: Animatable, A: Animation<T> + Sized> AnimationExt<T> for A {}

impl<Source: ?Sized, T: Animatable> Animation<T> for Box<Source>
where
    Source: Animation<T>,
{
    fn value(&self) -> &T {
        (**self).value()
    }

    fn state(&self) -> AnimationState {
        (**self).state()
    }

    fn duration(&self) -> Option<Duration> {
        (**self).duration()
    }

    fn tick(&mut self, delta: Duration) {
        (**self).tick(delta);
    }

    fn advance(&mut self, delta: Duration) -> Duration {
        (**self).advance(delta)
    }

    fn pause(&mut self) {
        (**self).pause();
    }

    fn resume(&mut self) {
        (**self).resume();
    }

    fn cancel(&mut self) {
        (**self).cancel();
    }

    fn seek(&mut self, progress: f32) {
        (**self).seek(progress);
    }

    fn finish(&mut self) {
        (**self).finish();
    }

    fn retarget(&mut self, target: &T) -> bool {
        (**self).retarget(target)
    }

    fn is_active(&self) -> bool {
        (**self).is_active()
    }

    fn set_rate(&mut self, rate: f64) {
        (**self).set_rate(rate);
    }

    fn into_value(self: Box<Self>) -> T {
        (*self).into_value()
    }
}
