use std::any::Any;

use crate::{
    runtime::{AnimationCommand, AnimationDyn},
    timing::Duration,
    traits::{Animatable, Animation, AnimationState},
};

enum AnimationStorage<T: Animatable> {
    Running(Box<dyn Animation<T>>),
    Settled { value: T, state: AnimationState },
    Transitioning,
}

pub(super) struct TypedAnimation<T: Animatable> {
    storage: AnimationStorage<T>,
}

impl<T: Animatable> TypedAnimation<T> {
    pub(super) fn new(animation: impl Animation<T>) -> Self {
        Self {
            storage: AnimationStorage::Running(Box::new(animation)),
        }
    }
}

impl<T: Animatable> AnimationDyn for TypedAnimation<T> {
    fn advance(&mut self, delta: Duration) {
        if let AnimationStorage::Running(animation) = &mut self.storage {
            animation.advance(delta);
        }
    }

    fn command(&mut self, command: AnimationCommand) {
        match &mut self.storage {
            AnimationStorage::Running(animation) => match command {
                AnimationCommand::Pause => animation.pause(),
                AnimationCommand::Resume => animation.resume(),
                AnimationCommand::Cancel => animation.cancel(),
                AnimationCommand::Seek(progress) => animation.seek(progress),
                AnimationCommand::Finish => animation.finish(),
            },
            AnimationStorage::Settled { state, .. } => {
                if command == AnimationCommand::Finish {
                    *state = AnimationState::Completed;
                }
            }
            AnimationStorage::Transitioning => {
                unreachable!("animation storage is only transitioning during compact")
            }
        }
    }

    fn compact(&mut self) {
        let AnimationStorage::Running(animation) = &self.storage else {
            return;
        };
        let state = animation.state();
        if !matches!(state, AnimationState::Completed | AnimationState::Canceled) {
            return;
        }

        let AnimationStorage::Running(animation) =
            std::mem::replace(&mut self.storage, AnimationStorage::Transitioning)
        else {
            unreachable!("animation storage was checked before compacting")
        };
        self.storage = AnimationStorage::Settled {
            value: animation.into_value(),
            state,
        };
    }

    fn is_active(&self) -> bool {
        match &self.storage {
            AnimationStorage::Running(animation) => animation.is_active(),
            AnimationStorage::Settled { .. } => false,
            AnimationStorage::Transitioning => {
                unreachable!("animation storage is only transitioning during compact")
            }
        }
    }

    fn state(&self) -> AnimationState {
        match &self.storage {
            AnimationStorage::Running(animation) => animation.state(),
            AnimationStorage::Settled { state, .. } => *state,
            AnimationStorage::Transitioning => {
                unreachable!("animation storage is only transitioning during compact")
            }
        }
    }

    fn value_any(&self) -> &dyn Any {
        match &self.storage {
            AnimationStorage::Running(animation) => animation.value(),
            AnimationStorage::Settled { value, .. } => value,
            AnimationStorage::Transitioning => {
                unreachable!("animation storage is only transitioning during compact")
            }
        }
    }

    fn value_type_name(&self) -> &'static str {
        std::any::type_name::<T>()
    }

    fn retarget_any(&mut self, target: &dyn Any) -> bool {
        let AnimationStorage::Running(animation) = &mut self.storage else {
            return false;
        };
        target
            .downcast_ref::<T>()
            .is_some_and(|target| animation.retarget(target))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use super::{AnimationDyn, AnimationStorage, TypedAnimation};
    use crate::{
        interpolate::InterpolationProgress,
        timing::Duration,
        traits::{Animation, AnimationState, Interpolate},
    };

    struct TrackedValue {
        value: i32,
        clone_count: Arc<AtomicUsize>,
    }

    impl Clone for TrackedValue {
        fn clone(&self) -> Self {
            self.clone_count.fetch_add(1, Ordering::Relaxed);
            Self {
                value: self.value,
                clone_count: Arc::clone(&self.clone_count),
            }
        }
    }

    impl Interpolate for TrackedValue {
        fn interpolate_progress(from: &Self, _to: &Self, _progress: InterpolationProgress) -> Self {
            from.clone()
        }
    }

    struct DropMarker(Arc<AtomicUsize>);

    impl Drop for DropMarker {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::Relaxed);
        }
    }

    struct MoveAnimation {
        value: TrackedValue,
        drop_marker: DropMarker,
    }

    impl Animation<TrackedValue> for MoveAnimation {
        fn value(&self) -> &TrackedValue {
            &self.value
        }

        fn state(&self) -> AnimationState {
            AnimationState::Completed
        }

        fn tick(&mut self, _delta: Duration) {}

        fn pause(&mut self) {}

        fn resume(&mut self) {}

        fn cancel(&mut self) {}

        fn seek(&mut self, _progress: f32) {}

        fn finish(&mut self) {}

        fn into_value(self: Box<Self>) -> TrackedValue {
            let Self { value, drop_marker } = *self;
            drop(drop_marker);
            value
        }
    }

    #[test]
    fn compact_moves_value_inline_and_drops_source_without_cloning() {
        let clone_count = Arc::new(AtomicUsize::new(0));
        let drop_count = Arc::new(AtomicUsize::new(0));
        let mut animation = TypedAnimation::new(MoveAnimation {
            value: TrackedValue {
                value: 42,
                clone_count: Arc::clone(&clone_count),
            },
            drop_marker: DropMarker(Arc::clone(&drop_count)),
        });

        animation.compact();

        assert!(matches!(
            animation.storage,
            AnimationStorage::Settled {
                state: AnimationState::Completed,
                ..
            }
        ));
        assert_eq!(
            animation
                .value_any()
                .downcast_ref::<TrackedValue>()
                .map(|value| value.value),
            Some(42)
        );
        assert_eq!(clone_count.load(Ordering::Relaxed), 0);
        assert_eq!(drop_count.load(Ordering::Relaxed), 1);
    }

    struct CloneAnimation {
        value: TrackedValue,
    }

    impl Animation<TrackedValue> for CloneAnimation {
        fn value(&self) -> &TrackedValue {
            &self.value
        }

        fn state(&self) -> AnimationState {
            AnimationState::Completed
        }

        fn tick(&mut self, _delta: Duration) {}

        fn pause(&mut self) {}

        fn resume(&mut self) {}

        fn cancel(&mut self) {}

        fn seek(&mut self, _progress: f32) {}

        fn finish(&mut self) {}
    }

    #[test]
    fn compact_uses_clone_fallback_for_custom_animations() {
        let clone_count = Arc::new(AtomicUsize::new(0));
        let mut animation = TypedAnimation::new(CloneAnimation {
            value: TrackedValue {
                value: 7,
                clone_count: Arc::clone(&clone_count),
            },
        });

        animation.compact();

        assert_eq!(
            animation
                .value_any()
                .downcast_ref::<TrackedValue>()
                .map(|value| value.value),
            Some(7)
        );
        assert_eq!(clone_count.load(Ordering::Relaxed), 1);
    }
}
