//! Runtime storage for active Iced-first animations.

mod clock;
mod entry;
mod handle;
mod policy;
mod registration;
mod registry;
mod source;
mod target;
#[cfg(test)]
mod tests;
mod tick;

pub use clock::{AnimationClock, SystemClock};
pub use entry::AnimationPlaybackState;
pub use handle::AnimationHandle;
pub use policy::TickPolicy;
pub use registration::AnimationRegistration;
pub use target::{AnimationTargetId, TargetedPropertySnapshot};
pub use tick::AnimationTick;

use crate::runtime::clock::TestClock;
use crate::runtime::{
    entry::ActiveAnimation, registry::AnimationRegistry, source::AnimationSource,
};
use crate::{keyframes::Keyframes, timeline::Timeline, timing::Duration};

/// Runtime state owned by an Iced application.
#[derive(Debug, Clone)]
pub struct AnimationRuntime<C = SystemClock> {
    registry: AnimationRegistry,
    clock: C,
    motion_policy: TickPolicy,
}

impl AnimationRuntime<SystemClock> {
    /// Creates an empty runtime using a monotonic system clock.
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(SystemClock::new())
    }
}

impl Default for AnimationRuntime<SystemClock> {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationRuntime<TestClock> {
    /// Creates an empty runtime with a deterministic test clock at zero.
    #[must_use]
    pub fn testing() -> Self {
        Self::with_clock(TestClock::new())
    }

    /// Returns mutable access to the runtime clock.
    ///
    /// This is primarily useful for deterministic tests and custom clock
    /// integrations.
    pub const fn clock_mut(&mut self) -> &mut TestClock {
        &mut self.clock
    }
}

impl<C: AnimationClock> AnimationRuntime<C> {
    /// Creates an empty runtime with a custom clock.
    #[must_use]
    pub fn with_clock(clock: C) -> Self {
        Self {
            registry: AnimationRegistry::new(),
            clock,
            motion_policy: TickPolicy::default(),
        }
    }

    /// Registers an erased animation source and returns its initial runtime output.
    pub(crate) fn register_target(
        &mut self,
        target: AnimationTargetId,
        source: impl Into<AnimationSource>,
    ) -> AnimationRegistration {
        let handle = self.registry.allocate_handle();
        let now = self.clock.now();
        let source = source.into();
        let initial_snapshot = source.sample_at(Duration::ZERO);
        let mut entry = ActiveAnimation::new(handle, target, source, now);

        entry.set_last_snapshot(initial_snapshot.clone());

        if entry.source().total_duration() == Some(Duration::ZERO) {
            let completion_snapshot = entry.source().completion_snapshot();

            entry.set_last_snapshot(completion_snapshot);
            entry.mark_completed(now);
        }

        let registration = AnimationRegistration::from_entry(&entry);
        self.registry.insert(target, entry);

        registration
    }

    /// Registers keyframes and returns their initial runtime output.
    pub fn register_keyframes(
        &mut self,
        target: AnimationTargetId,
        keyframes: Keyframes,
    ) -> AnimationRegistration {
        self.register_target(target, keyframes)
    }

    /// Registers a timeline and returns its initial runtime output.
    pub fn register_timeline(
        &mut self,
        target: AnimationTargetId,
        timeline: Timeline,
    ) -> AnimationRegistration {
        self.register_target(target, timeline)
    }

    /// Advances active animations and returns a view-ready aggregated snapshot.
    pub fn tick(&mut self) -> AnimationTick {
        let now = self.clock.now();

        tick::tick_registry(&mut self.registry, now)
    }

    /// Cancels and removes all active animations registered for `target`.
    pub fn cancel_target(&mut self, target: AnimationTargetId) {
        self.registry.cancel_target(target);
    }

    /// Seeks all active animations registered for `target`.
    pub fn seek_target(&mut self, target: AnimationTargetId, pos: Duration) {
        self.registry.seek_target(target, pos, self.clock.now());
    }

    /// Pauses all active animations registered for `target`.
    pub fn pause_target(&mut self, target: AnimationTargetId) {
        self.registry.pause_target(target, self.clock.now());
    }

    /// Cancels and removes one animation when `handle` belongs to `target`.
    ///
    /// Returns `true` when an entry was removed.
    pub fn cancel(&mut self, target: AnimationTargetId, handle: AnimationHandle) -> bool {
        self.registry.cancel(target, handle)
    }

    /// Seeks one animation when `handle` belongs to `target`.
    ///
    /// Returns `true` when an entry was found and updated.
    pub fn seek(
        &mut self,
        target: AnimationTargetId,
        handle: AnimationHandle,
        pos: Duration,
    ) -> bool {
        self.registry.seek(target, handle, pos, self.clock.now())
    }

    /// Pauses one animation when `handle` belongs to `target`.
    ///
    /// Returns `true` when an entry was found and updated.
    pub fn pause(&mut self, target: AnimationTargetId, handle: AnimationHandle) -> bool {
        self.registry.pause(target, handle, self.clock.now())
    }
}

impl<C> AnimationRuntime<C> {
    /// Returns the runtime clock.
    #[must_use]
    pub const fn clock(&self) -> &C {
        &self.clock
    }

    /// Returns the current motion policy.
    #[must_use]
    pub const fn motion_policy(&self) -> TickPolicy {
        self.motion_policy
    }

    /// Replaces the current motion policy.
    pub const fn set_motion_policy(&mut self, motion_policy: TickPolicy) {
        self.motion_policy = motion_policy;
    }

    /// Returns the number of active animation entries.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.registry.active_count()
    }

    /// Returns whether the runtime has no active animation entries.
    #[must_use]
    pub fn is_idle(&self) -> bool {
        self.active_count() == 0
    }

    /// Returns whether the runtime has entries that should receive animation ticks.
    #[must_use]
    pub fn should_tick(&self) -> bool {
        self.registry
            .entries()
            .iter()
            .any(ActiveAnimation::needs_tick)
    }

    /// Returns whether an Iced subscription should keep producing animation ticks.
    #[must_use]
    pub fn should_subscribe(&self) -> bool {
        self.should_tick()
    }
}
