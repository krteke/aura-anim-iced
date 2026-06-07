use std::any::Any;

use crate::{
    Animatable, Animation, AnimationCommand, AnimationState,
    runtime::{AnimationDyn, Settled},
    timing::Duration,
};

pub(super) struct TypedAnimation<T: Animatable> {
    animation: Box<dyn Animation<T>>,
}

impl<T: Animatable> TypedAnimation<T> {
    pub(super) fn new(animation: impl Animation<T>) -> Self {
        Self {
            animation: Box::new(animation),
        }
    }
}

impl<T: Animatable> AnimationDyn for TypedAnimation<T> {
    fn advance(&mut self, delta: Duration) {
        self.animation.advance(delta);
    }

    fn command(&mut self, command: AnimationCommand) {
        match command {
            AnimationCommand::Pause => self.animation.pause(),
            AnimationCommand::Resume => self.animation.resume(),
            AnimationCommand::Cancel => self.animation.cancel(),
            AnimationCommand::Seek(progress) => self.animation.seek(progress),
            AnimationCommand::Finish => self.animation.finish(),
        }
    }

    fn compact(&mut self) {
        let state = self.animation.state();
        if matches!(state, AnimationState::Completed | AnimationState::Canceled) {
            self.animation = Box::new(Settled::new(self.animation.value().clone(), state));
        }
    }

    fn is_active(&self) -> bool {
        self.animation.is_active()
    }

    fn state(&self) -> AnimationState {
        self.animation.state()
    }

    fn value_any(&self) -> &dyn Any {
        self.animation.value()
    }

    fn retarget_any(&mut self, target: &dyn Any) -> bool {
        target
            .downcast_ref::<T>()
            .is_some_and(|target| self.animation.retarget(target))
    }
}
