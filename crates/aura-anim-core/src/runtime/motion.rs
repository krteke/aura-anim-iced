use std::marker::PhantomData;

use crate::{Animatable, Animation, AnimationCommand, AnimationState, MotionRuntime};

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
    ///
    /// Returns `false` when the handle is no longer valid.
    pub fn transition_to(self, target: T, runtime: &mut MotionRuntime) -> bool {
        runtime.transition_to(self, target)
    }

    /// Replaces this motion's current animation.
    ///
    /// Returns `false` when the handle is no longer valid.
    pub fn play(self, animation: impl Animation<T>, runtime: &mut MotionRuntime) -> bool {
        runtime.play(self, animation)
    }

    /// Clones and returns the current value.
    ///
    /// # Panics
    ///
    /// Panics when the handle is no longer valid.
    #[must_use]
    pub fn value(self, runtime: &MotionRuntime) -> T {
        runtime
            .value(self)
            .cloned()
            .expect("motion handle is no longer valid")
    }

    /// Borrows the current value.
    ///
    /// # Panics
    ///
    /// Panics when the handle is no longer valid.
    #[must_use]
    pub fn value_ref(self, runtime: &MotionRuntime) -> &T {
        runtime
            .value(self)
            .expect("motion handle is no longer valid")
    }

    /// Borrows the current value when the handle is valid.
    #[must_use]
    pub fn try_value(self, runtime: &MotionRuntime) -> Option<&T> {
        runtime.value(self)
    }

    /// Returns the current state when the handle is valid.
    #[must_use]
    pub fn state(self, runtime: &MotionRuntime) -> Option<AnimationState> {
        runtime.state(self)
    }

    /// Returns whether the motion is active.
    #[must_use]
    pub fn is_active(self, runtime: &MotionRuntime) -> bool {
        runtime.is_active(self)
    }

    /// Returns whether the motion completed.
    #[must_use]
    pub fn is_completed(self, runtime: &MotionRuntime) -> bool {
        self.state(runtime) == Some(AnimationState::Completed)
    }

    /// Pauses the motion if the handle is valid.
    pub fn pause(self, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Pause)
    }

    /// Resumes the motion if the handle is valid.
    pub fn resume(self, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Resume)
    }

    /// Cancels the motion if the handle is valid.
    pub fn cancel(self, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Cancel)
    }

    /// Seeks the motion to normalized progress if the handle is valid.
    pub fn seek(self, progress: f32, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Seek(progress))
    }

    /// Moves the motion to completion if the handle is valid.
    pub fn finish(self, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Finish)
    }

    /// Removes the motion from the runtime.
    pub fn remove(self, runtime: &mut MotionRuntime) -> bool {
        runtime.remove(self)
    }
}
