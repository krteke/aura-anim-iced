use std::marker::PhantomData;

use crate::{
    runtime::{AnimationCommand, MotionError, MotionRuntime},
    traits::{Animatable, AnimationState, IntoMotionAnimation},
};

use super::{MotionId, PlaybackId};

/// A typed handle to an animation stored in a [`MotionRuntime`].
///
/// Handles become invalid after their animation is removed.
#[must_use = "a motion handle is required to access the runtime-managed animation"]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Motion<T> {
    id: MotionId,
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Motion<T> {}

impl<T> Clone for Motion<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Motion<T> {
    pub(super) fn new(id: MotionId, marker: PhantomData<fn() -> T>) -> Self {
        Self { id, marker }
    }

    pub(super) const fn id(self) -> MotionId {
        self.id
    }

    /// Returns the untyped ID for this motion lifecycle.
    #[must_use]
    pub const fn motion_id(self) -> MotionId {
        self.id
    }
}

impl<T: Animatable> Motion<T> {
    /// Transitions this motion toward `target`.
    pub fn transition_to(self, target: T, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        runtime.transition_to(self, target)
    }

    /// Transitions this motion and returns the new playback ID.
    pub fn transition_to_tracked(
        self,
        target: T,
        runtime: &mut MotionRuntime,
    ) -> Result<PlaybackId, MotionError> {
        runtime.transition_to_tracked(self, target)
    }

    /// Replaces this motion's current animation.
    pub fn play<P, Kind>(self, playback: P, runtime: &mut MotionRuntime) -> Result<(), MotionError>
    where
        P: IntoMotionAnimation<T, Kind>,
    {
        runtime.play(self, playback)
    }

    /// Replaces this motion's current animation and returns its playback ID.
    pub fn play_tracked<P, Kind>(
        self,
        playback: P,
        runtime: &mut MotionRuntime,
    ) -> Result<PlaybackId, MotionError>
    where
        P: IntoMotionAnimation<T, Kind>,
    {
        runtime.play_tracked(self, playback)
    }

    /// Clones and returns the current value.
    pub fn value(self, runtime: &MotionRuntime) -> Result<T, MotionError> {
        runtime.value(self).cloned()
    }

    /// Borrows the current value.
    pub fn value_ref(self, runtime: &MotionRuntime) -> Result<&T, MotionError> {
        runtime.value(self)
    }

    /// Returns the current lifecycle state.
    pub fn state(self, runtime: &MotionRuntime) -> Result<AnimationState, MotionError> {
        runtime.state(self)
    }

    /// Returns the ID of this motion's current playback.
    pub fn playback(self, runtime: &MotionRuntime) -> Result<PlaybackId, MotionError> {
        runtime.playback(self)
    }

    /// Returns whether the motion is active.
    pub fn is_active(self, runtime: &MotionRuntime) -> Result<bool, MotionError> {
        runtime.is_active(self)
    }

    /// Returns whether the motion completed.
    pub fn is_completed(self, runtime: &MotionRuntime) -> Result<bool, MotionError> {
        self.state(runtime)
            .map(|state| state == AnimationState::Completed)
    }

    /// Pauses the motion.
    pub fn pause(self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        runtime.command(self, AnimationCommand::Pause)
    }

    /// Resumes the motion.
    pub fn resume(self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        runtime.command(self, AnimationCommand::Resume)
    }

    /// Cancels the motion.
    pub fn cancel(self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        runtime.command(self, AnimationCommand::Cancel)
    }

    /// Seeks the motion to normalized progress.
    pub fn seek(self, progress: f32, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        runtime.command(self, AnimationCommand::Seek(progress))
    }

    /// Moves the motion to completion.
    pub fn finish(self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        runtime.command(self, AnimationCommand::Finish)
    }

    /// Removes the motion from the runtime.
    pub fn remove(self, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        runtime.remove(self)
    }
}
