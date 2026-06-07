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

    pub fn transition_to(self, target: T, runtime: &mut MotionRuntime) -> bool {
        runtime.transition_to(self, target)
    }

    pub fn play(self, animation: impl Animation<T>, runtime: &mut MotionRuntime) -> bool {
        runtime.play(self, animation)
    }

    pub fn value(self, runtime: &MotionRuntime) -> T {
        runtime
            .value(self)
            .cloned()
            .expect("motion handle is no longer valid")
    }

    pub fn value_ref(self, runtime: &MotionRuntime) -> &T {
        runtime
            .value(self)
            .expect("motion handle is no longer valid")
    }

    pub fn try_value(self, runtime: &MotionRuntime) -> Option<&T> {
        runtime.value(self)
    }

    pub fn state(self, runtime: &MotionRuntime) -> Option<AnimationState> {
        runtime.state(self)
    }

    pub fn is_active(self, runtime: &MotionRuntime) -> bool {
        runtime.is_active(self)
    }

    pub fn is_completed(self, runtime: &MotionRuntime) -> bool {
        self.state(runtime) == Some(AnimationState::Completed)
    }

    pub fn pause(self, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Pause)
    }

    pub fn resume(self, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Resume)
    }

    pub fn cancel(self, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Cancel)
    }

    pub fn seek(self, progress: f32, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Seek(progress))
    }

    pub fn finish(self, runtime: &mut MotionRuntime) -> bool {
        runtime.command(self, AnimationCommand::Finish)
    }

    pub fn remove(self, runtime: &mut MotionRuntime) -> bool {
        runtime.remove(self)
    }
}
