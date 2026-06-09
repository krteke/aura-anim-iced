//! Runtime storage, typed handles, and animation commands.

use std::marker::PhantomData;

use crate::{
    runtime::{
        anim_dyn::AnimationDyn, motion::RawMotionId, settled::Settled, typed::TypedAnimation,
    },
    timing::{Duration, Timing},
    traits::{Animatable, Animation, AnimationState},
    tween::Tween,
};

mod anim_dyn;
mod command;
mod error;
mod motion;
mod settled;
mod typed;

pub use command::AnimationCommand;
pub use error::MotionError;
pub use motion::Motion;

/// Controls whether the runtime retains an animation after it settles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetainPolicy {
    /// Keep the animation and its final value until explicitly removed.
    #[default]
    Keep,
    /// Remove the animation after it completes or is canceled.
    DropWhenSettled,
}

struct AnimationSlot {
    generation: u64,
    transition: Timing,
    retain_policy: RetainPolicy,
    animation: Option<Box<dyn AnimationDyn>>,
    active: bool,
    queued: bool,
}

/// Stores animations and advances their active values.
///
/// # Examples
///
/// ```
/// use aura_anim_core::{MotionRuntime, timing::Timing};
/// use std::time::Duration;
///
/// let mut runtime = MotionRuntime::new();
/// let opacity = runtime.motion_with(0.0_f32, Timing::new(100.0));
///
/// opacity.transition_to(1.0, &mut runtime).unwrap();
/// runtime.tick(Duration::from_millis(50));
///
/// assert_eq!(opacity.value(&runtime).unwrap(), 0.5);
/// assert!(opacity.is_active(&runtime).unwrap());
/// ```
#[derive(Default)]
pub struct MotionRuntime {
    slots: Vec<AnimationSlot>,
    free: Vec<usize>,
    active: Vec<RawMotionId>,
    next_active: Vec<RawMotionId>,
    active_count: usize,
    motion_count: usize,
    last_tick: Option<std::time::Instant>,
}

impl MotionRuntime {
    /// Creates an empty motion runtime.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts an idle motion with the default transition timing.
    pub fn motion<T: Animatable>(&mut self, initial: T) -> Motion<T> {
        self.motion_with(initial, Timing::new(200.0))
    }

    /// Inserts an idle motion with the provided transition timing.
    pub fn motion_with<T: Animatable>(&mut self, initial: T, timing: Timing) -> Motion<T> {
        self.insert(Tween::with_timing(initial, timing), timing)
    }

    /// Inserts an animation that remains stored after it settles.
    pub fn insert<T: Animatable>(
        &mut self,
        animation: impl Animation<T>,
        transition: Timing,
    ) -> Motion<T> {
        self.insert_with_policy(animation, transition, RetainPolicy::Keep)
    }

    /// Inserts an animation that is removed after it settles.
    pub fn play_once<T: Animatable>(&mut self, animation: impl Animation<T>) -> Motion<T> {
        self.insert_with_policy(animation, Timing::default(), RetainPolicy::DropWhenSettled)
    }

    /// Inserts an animation with transition timing and a retention policy.
    pub fn insert_with_policy<T: Animatable>(
        &mut self,
        animation: impl Animation<T>,
        transition: Timing,
        retain_policy: RetainPolicy,
    ) -> Motion<T> {
        let mut animation = TypedAnimation::new(animation);
        animation.compact();
        let animation: Box<dyn AnimationDyn> = Box::new(animation);
        let (id, reused_slot) = if let Some(slot_index) = self.free.pop() {
            let slot = &mut self.slots[slot_index];
            slot.generation = slot.generation.wrapping_add(1);
            slot.transition = transition;
            slot.retain_policy = retain_policy;
            slot.animation = Some(animation);
            slot.active = false;
            slot.queued = false;
            (RawMotionId::new(slot_index, slot.generation), true)
        } else {
            let slot = self.slots.len();
            self.slots.push(AnimationSlot {
                generation: 0,
                transition,
                retain_policy,
                animation: Some(animation),
                active: false,
                queued: false,
            });
            (RawMotionId::new(slot, 0), false)
        };

        self.motion_count += 1;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::runtime",
            slot = id.slot(),
            generation = id.generation(),
            value_type = std::any::type_name::<T>(),
            ?retain_policy,
            reused_slot,
            motion_count = self.motion_count,
            "inserted motion"
        );
        #[cfg(not(feature = "tracing"))]
        let _ = reused_slot;
        let motion = Motion::new(id, PhantomData);
        if self.animation(id).is_ok_and(AnimationDyn::is_active) {
            self.activate(id);
        } else if retain_policy == RetainPolicy::DropWhenSettled
            && self.animation(id).is_ok_and(|animation| {
                matches!(
                    animation.state(),
                    AnimationState::Completed | AnimationState::Canceled
                )
            })
        {
            debug_assert!(self.remove(motion).is_ok());
        }
        motion
    }

    /// Advances every active animation by `delta`.
    pub fn tick(&mut self, delta: impl Into<Duration>) {
        let delta = delta.into();
        #[cfg(feature = "tracing")]
        tracing::trace!(
            target: "aura_anim::runtime",
            delta_ms = delta.as_millis(),
            active_count = self.active_count,
            queued_count = self.active.len(),
            "ticking motions"
        );
        self.next_active.clear();

        for index in 0..self.active.len() {
            let id = self.active[index];
            let Some(slot) = self.slots.get_mut(id.slot()) else {
                continue;
            };
            if slot.generation != id.generation() {
                continue;
            }
            slot.queued = false;
            if !slot.active {
                continue;
            }
            let Some(animation) = slot.animation.as_mut() else {
                continue;
            };

            animation.advance(delta);
            if animation.is_active() {
                self.next_active.push(id);
                slot.queued = true;
            } else {
                slot.active = false;
                self.active_count = self.active_count.saturating_sub(1);
                if slot.retain_policy == RetainPolicy::DropWhenSettled
                    && matches!(
                        animation.state(),
                        AnimationState::Completed | AnimationState::Canceled
                    )
                {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(
                        target: "aura_anim::runtime",
                        slot = id.slot(),
                        generation = id.generation(),
                        state = ?animation.state(),
                        "dropping settled transient motion"
                    );
                    slot.animation = None;
                    self.motion_count = self.motion_count.saturating_sub(1);
                    self.free.push(id.slot());
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::debug!(
                        target: "aura_anim::runtime",
                        slot = id.slot(),
                        generation = id.generation(),
                        state = ?animation.state(),
                        "motion settled"
                    );
                    animation.compact();
                }
            }
        }

        std::mem::swap(&mut self.active, &mut self.next_active);
        if self.active_count == 0 {
            self.active.clear();
            self.next_active.clear();
            self.last_tick = None;
        }
        #[cfg(feature = "tracing")]
        tracing::trace!(
            target: "aura_anim::runtime",
            active_count = self.active_count,
            motion_count = self.motion_count,
            "finished ticking motions"
        );
    }

    /// Advances active animations using elapsed wall-clock time.
    ///
    /// The first call establishes the clock origin and advances by zero.
    pub fn tick_at(&mut self, now: std::time::Instant) {
        let delta = self.last_tick.map_or(std::time::Duration::ZERO, |last| {
            now.saturating_duration_since(last)
        });
        self.last_tick = Some(now);
        self.tick(delta);
    }

    /// Returns the number of animations currently marked active.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.active_count
    }

    /// Returns whether at least one animation is active.
    #[must_use]
    pub fn has_active(&self) -> bool {
        self.active_count > 0
    }

    /// Returns the number of stored animations.
    #[must_use]
    pub fn motion_count(&self) -> usize {
        self.motion_count
    }

    /// Returns the allocated capacity of the runtime slot storage.
    #[must_use]
    pub fn slot_capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Releases unused slot and queue capacity.
    pub fn shrink_to_fit(&mut self) {
        while self
            .slots
            .last()
            .is_some_and(|slot| slot.animation.is_none())
        {
            self.slots.pop();
        }
        self.free.retain(|slot| *slot < self.slots.len());
        self.slots.shrink_to_fit();
        self.free.shrink_to_fit();
        self.active.shrink_to_fit();
        self.next_active.shrink_to_fit();
    }

    /// Returns the current value for `motion`.
    pub fn value<T: Animatable>(&self, motion: Motion<T>) -> Result<&T, MotionError> {
        let animation = self.animation(motion.id())?;
        animation.value_any().downcast_ref::<T>().ok_or_else(|| {
            let error = MotionError::TypeMismatch {
                expected: std::any::type_name::<T>(),
                actual: animation.value_type_name(),
            };
            #[cfg(feature = "tracing")]
            trace_motion_error(motion.id(), &error);
            error
        })
    }

    /// Returns the lifecycle state for `motion`.
    pub fn state<T: Animatable>(&self, motion: Motion<T>) -> Result<AnimationState, MotionError> {
        self.animation(motion.id()).map(AnimationDyn::state)
    }

    /// Returns whether `motion` is active.
    pub fn is_active<T: Animatable>(&self, motion: Motion<T>) -> Result<bool, MotionError> {
        self.animation(motion.id()).map(AnimationDyn::is_active)
    }

    /// Transitions `motion` toward `target`.
    pub fn transition_to<T: Animatable>(
        &mut self,
        motion: Motion<T>,
        target: T,
    ) -> Result<(), MotionError> {
        if self.animation_mut(motion.id())?.retarget_any(&target) {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                target: "aura_anim::runtime",
                slot = motion.id().slot(),
                generation = motion.id().generation(),
                value_type = std::any::type_name::<T>(),
                "retargeted motion in place"
            );
            self.activate(motion.id());
            return Ok(());
        }

        let current = self.value(motion)?.clone();
        let transition = self.slots[motion.id().slot()].transition;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::runtime",
            slot = motion.id().slot(),
            generation = motion.id().generation(),
            value_type = std::any::type_name::<T>(),
            "replacing motion with transition tween"
        );
        self.replace(
            motion.id(),
            TypedAnimation::new(Tween::between(current, target, transition)),
        )
    }

    /// Replaces the animation associated with `motion`.
    pub fn play<T: Animatable>(
        &mut self,
        motion: Motion<T>,
        animation: impl Animation<T>,
    ) -> Result<(), MotionError> {
        self.value(motion)?;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::runtime",
            slot = motion.id().slot(),
            generation = motion.id().generation(),
            value_type = std::any::type_name::<T>(),
            "playing replacement animation"
        );
        self.replace(motion.id(), TypedAnimation::new(animation))
    }

    /// Applies a command to `motion`.
    pub fn command<T: Animatable>(
        &mut self,
        motion: Motion<T>,
        command: AnimationCommand,
    ) -> Result<(), MotionError> {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::runtime",
            slot = motion.id().slot(),
            generation = motion.id().generation(),
            ?command,
            "applying motion command"
        );
        let (active, state) = {
            let animation = self.animation_mut(motion.id())?;
            animation.command(command);
            (animation.is_active(), animation.state())
        };

        if active {
            self.activate(motion.id());
        } else {
            self.deactivate(motion.id());
            if self.slots[motion.id().slot()].retain_policy == RetainPolicy::DropWhenSettled
                && matches!(state, AnimationState::Completed | AnimationState::Canceled)
            {
                self.remove(motion)?;
            } else {
                self.animation_mut(motion.id())?.compact();
            }
        }
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::runtime",
            slot = motion.id().slot(),
            generation = motion.id().generation(),
            ?state,
            active,
            "applied motion command"
        );
        Ok(())
    }

    /// Removes the animation referenced by `motion`.
    pub fn remove<T: Animatable>(&mut self, motion: Motion<T>) -> Result<(), MotionError> {
        self.animation(motion.id())?;
        let slot = &mut self.slots[motion.id().slot()];

        let was_active = slot.active;
        slot.animation = None;
        slot.active = false;
        slot.queued = false;
        if was_active {
            self.active_count = self.active_count.saturating_sub(1);
        }
        self.motion_count = self.motion_count.saturating_sub(1);
        self.free.push(motion.id().slot());

        let slot_idx = motion.id().slot();
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::runtime",
            slot = slot_idx,
            generation = motion.id().generation(),
            was_active,
            motion_count = self.motion_count,
            "removed motion"
        );
        self.active.retain(|id| id.slot() != slot_idx);
        self.next_active.retain(|id| id.slot() != slot_idx);

        if self.active_count == 0 {
            self.active.clear();
            self.next_active.clear();
            self.last_tick = None;
        }
        Ok(())
    }

    fn animation(&self, id: RawMotionId) -> Result<&dyn AnimationDyn, MotionError> {
        let Some(slot) = self.slots.get(id.slot()) else {
            let error = MotionError::SlotOutOfBounds { slot: id.slot() };
            #[cfg(feature = "tracing")]
            trace_motion_error(id, &error);
            return Err(error);
        };
        if slot.generation != id.generation() {
            let error = MotionError::StaleHandle {
                slot: id.slot(),
                handle_generation: id.generation(),
                actual_generation: slot.generation,
            };
            #[cfg(feature = "tracing")]
            trace_motion_error(id, &error);
            return Err(error);
        }
        slot.animation.as_deref().ok_or_else(|| {
            let error = MotionError::Removed { slot: id.slot() };
            #[cfg(feature = "tracing")]
            trace_motion_error(id, &error);
            error
        })
    }

    fn animation_mut(
        &mut self,
        id: RawMotionId,
    ) -> Result<&mut (dyn AnimationDyn + '_), MotionError> {
        let Some(slot) = self.slots.get_mut(id.slot()) else {
            let error = MotionError::SlotOutOfBounds { slot: id.slot() };
            #[cfg(feature = "tracing")]
            trace_motion_error(id, &error);
            return Err(error);
        };
        if slot.generation != id.generation() {
            let error = MotionError::StaleHandle {
                slot: id.slot(),
                handle_generation: id.generation(),
                actual_generation: slot.generation,
            };
            #[cfg(feature = "tracing")]
            trace_motion_error(id, &error);
            return Err(error);
        }
        if let Some(animation) = slot.animation.as_mut() {
            Ok(animation.as_mut())
        } else {
            let error = MotionError::Removed { slot: id.slot() };
            #[cfg(feature = "tracing")]
            trace_motion_error(id, &error);
            Err(error)
        }
    }

    fn replace(
        &mut self,
        id: RawMotionId,
        animation: TypedAnimation<impl Animatable>,
    ) -> Result<(), MotionError> {
        self.animation(id)?;
        let slot = &mut self.slots[id.slot()];
        let mut animation = animation;
        animation.compact();
        slot.animation = Some(Box::new(animation));
        if slot
            .animation
            .as_deref()
            .is_some_and(AnimationDyn::is_active)
        {
            self.activate(id);
        } else {
            self.deactivate(id);
        }
        Ok(())
    }

    fn activate(&mut self, id: RawMotionId) {
        let Some(slot) = self.slots.get_mut(id.slot()) else {
            return;
        };
        if slot.generation != id.generation() || slot.animation.is_none() {
            return;
        }
        if self.active_count == 0 {
            self.last_tick = None;
        }
        if !slot.active {
            slot.active = true;
            self.active_count += 1;
        }
        if !slot.queued {
            slot.queued = true;
            self.active.push(id);
        }
    }

    fn deactivate(&mut self, id: RawMotionId) {
        let Some(slot) = self.slots.get_mut(id.slot()) else {
            return;
        };
        if slot.generation != id.generation() || !slot.active {
            return;
        }
        slot.active = false;
        self.active_count = self.active_count.saturating_sub(1);
        if self.active_count == 0 {
            for active in &self.active {
                if let Some(slot) = self.slots.get_mut(active.slot())
                    && slot.generation == active.generation()
                {
                    slot.queued = false;
                }
            }
            self.active.clear();
            self.next_active.clear();
            self.last_tick = None;
        }
    }
}

#[cfg(feature = "tracing")]
fn trace_motion_error(id: RawMotionId, error: &MotionError) {
    tracing::debug!(
        target: "aura_anim::runtime",
        slot = id.slot(),
        generation = id.generation(),
        error = %error,
        "motion lookup failed"
    );
}
