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
mod motion;
mod settled;
mod typed;

pub use command::AnimationCommand;
pub use motion::Motion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetainPolicy {
    #[default]
    Keep,
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn motion<T: Animatable>(&mut self, initial: T) -> Motion<T> {
        self.motion_with(initial, Timing::new(200.0))
    }

    pub fn motion_with<T: Animatable>(&mut self, initial: T, timing: Timing) -> Motion<T> {
        self.insert(Tween::with_timing(initial, timing), timing)
    }

    pub fn insert<T: Animatable>(
        &mut self,
        animation: impl Animation<T>,
        transition: Timing,
    ) -> Motion<T> {
        self.insert_with_policy(animation, transition, RetainPolicy::Keep)
    }

    pub fn play_once<T: Animatable>(&mut self, animation: impl Animation<T>) -> Motion<T> {
        self.insert_with_policy(animation, Timing::default(), RetainPolicy::DropWhenSettled)
    }

    pub fn insert_with_policy<T: Animatable>(
        &mut self,
        animation: impl Animation<T>,
        transition: Timing,
        retain_policy: RetainPolicy,
    ) -> Motion<T> {
        let mut animation = TypedAnimation::new(animation);
        animation.compact();
        let animation: Box<dyn AnimationDyn> = Box::new(animation);
        let id = if let Some(slot_index) = self.free.pop() {
            let slot = &mut self.slots[slot_index];
            slot.transition = transition;
            slot.retain_policy = retain_policy;
            slot.animation = Some(animation);
            slot.active = false;
            slot.queued = false;
            RawMotionId::new(slot_index, slot.generation)
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
            RawMotionId::new(slot, 0)
        };

        self.motion_count += 1;
        let motion = Motion::new(id, PhantomData);
        if self.animation(id).is_some_and(AnimationDyn::is_active) {
            self.activate(id);
        } else if retain_policy == RetainPolicy::DropWhenSettled
            && self.animation(id).is_some_and(|animation| {
                matches!(
                    animation.state(),
                    AnimationState::Completed | AnimationState::Canceled
                )
            })
        {
            self.remove(motion);
        }
        motion
    }

    pub fn tick(&mut self, delta: impl Into<Duration>) {
        let delta = delta.into();
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
                    slot.animation = None;
                    slot.generation = slot.generation.wrapping_add(1);
                    self.motion_count = self.motion_count.saturating_sub(1);
                    self.free.push(id.slot());
                } else {
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
    }

    pub fn tick_at(&mut self, now: std::time::Instant) {
        let delta = self.last_tick.map_or(std::time::Duration::ZERO, |last| {
            now.saturating_duration_since(last)
        });
        self.last_tick = Some(now);
        self.tick(delta);
    }

    pub fn active_count(&self) -> usize {
        self.active_count
    }

    pub fn has_active(&self) -> bool {
        self.active_count > 0
    }

    pub fn motion_count(&self) -> usize {
        self.motion_count
    }

    pub fn slot_capacity(&self) -> usize {
        self.slots.capacity()
    }

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

    pub fn value<T: Animatable>(&self, motion: Motion<T>) -> Option<&T> {
        self.animation(motion.id())?.value_any().downcast_ref::<T>()
    }

    pub fn state<T: Animatable>(&self, motion: Motion<T>) -> Option<AnimationState> {
        self.animation(motion.id()).map(AnimationDyn::state)
    }

    pub fn is_active<T: Animatable>(&self, motion: Motion<T>) -> bool {
        self.animation(motion.id())
            .is_some_and(AnimationDyn::is_active)
    }

    pub fn transition_to<T: Animatable>(&mut self, motion: Motion<T>, target: T) -> bool {
        if self
            .animation_mut(motion.id())
            .is_some_and(|animation| animation.retarget_any(&target))
        {
            self.activate(motion.id());
            return true;
        }

        let Some(current) = self.value(motion).cloned() else {
            return false;
        };
        let transition = self.slots[motion.id().slot()].transition;
        self.replace(
            motion.id(),
            TypedAnimation::new(Tween::between(current, target, transition)),
        );
        true
    }

    pub fn play<T: Animatable>(&mut self, motion: Motion<T>, animation: impl Animation<T>) -> bool {
        if self.value(motion).is_none() {
            return false;
        }

        self.replace(motion.id(), TypedAnimation::new(animation));
        true
    }

    pub fn command<T: Animatable>(&mut self, motion: Motion<T>, command: AnimationCommand) -> bool {
        let (active, state) = {
            let Some(animation) = self.animation_mut(motion.id()) else {
                return false;
            };
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
                self.remove(motion);
            } else if let Some(animation) = self.animation_mut(motion.id()) {
                animation.compact();
            }
        }
        true
    }

    pub fn remove<T: Animatable>(&mut self, motion: Motion<T>) -> bool {
        let Some(slot) = self.slots.get_mut(motion.id().slot()) else {
            return false;
        };
        if slot.generation != motion.id().generation() || slot.animation.is_none() {
            return false;
        }

        let was_active = slot.active;
        slot.animation = None;
        slot.active = false;
        slot.queued = false;
        slot.generation = slot.generation.wrapping_add(1);
        if was_active {
            self.active_count = self.active_count.saturating_sub(1);
        }
        self.motion_count = self.motion_count.saturating_sub(1);
        self.free.push(motion.id().slot());

        let slot_idx = motion.id().slot();
        let new_gen = slot.generation;
        self.active
            .retain(|id| id.slot() != slot_idx || id.generation() == new_gen);
        self.next_active
            .retain(|id| id.slot() != slot_idx || id.generation() == new_gen);

        if self.active_count == 0 {
            self.active.clear();
            self.next_active.clear();
            self.last_tick = None;
        }
        true
    }

    fn animation(&self, id: RawMotionId) -> Option<&dyn AnimationDyn> {
        let slot = self.slots.get(id.slot())?;
        if slot.generation != id.generation() {
            return None;
        }
        slot.animation.as_deref()
    }

    fn animation_mut(&mut self, id: RawMotionId) -> Option<&mut (dyn AnimationDyn + '_)> {
        let slot = self.slots.get_mut(id.slot())?;
        if slot.generation != id.generation() {
            return None;
        }
        match slot.animation.as_mut() {
            Some(animation) => Some(animation.as_mut()),
            None => None,
        }
    }

    fn replace(&mut self, id: RawMotionId, animation: TypedAnimation<impl Animatable>) {
        let Some(slot) = self.slots.get_mut(id.slot()) else {
            return;
        };
        if slot.generation != id.generation() {
            return;
        }
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
