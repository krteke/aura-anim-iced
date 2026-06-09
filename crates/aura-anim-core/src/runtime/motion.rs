use std::marker::PhantomData;

use crate::{Animatable, Animation, AnimationCommand, AnimationState, MotionError, MotionRuntime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct RawMotionId {
    slot: usize,
    generation: u64,
}

impl RawMotionId {
    pub(super) fn new(slot: usize, generation: u64) -> Self {
        Self { slot, generation }
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn slot(&self) -> usize {
        self.slot
    }
}

/// A typed handle to an animation stored in a [`MotionRuntime`].
///
/// Handles become invalid after their animation is removed.
#[must_use = "a motion handle is required to access the runtime-managed animation"]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Motion<T> {
    id: RawMotionId,
    marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Motion<T> {}

impl<T> Clone for Motion<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Animatable> Motion<T> {
    pub(super) fn new(id: RawMotionId, marker: PhantomData<fn() -> T>) -> Self {
        Self { id, marker }
    }

    pub(super) fn id(&self) -> RawMotionId {
        self.id
    }

    /// Transitions this motion toward `target`.
    pub fn transition_to(self, target: T, runtime: &mut MotionRuntime) -> Result<(), MotionError> {
        runtime.transition_to(self, target)
    }

    /// Replaces this motion's current animation.
    pub fn play(
        self,
        animation: impl Animation<T>,
        runtime: &mut MotionRuntime,
    ) -> Result<(), MotionError> {
        runtime.play(self, animation)
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
