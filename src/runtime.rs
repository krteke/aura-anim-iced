//! Runtime storage for active Iced-first animations.

mod clock;
mod entry;
mod handle;
mod policy;
mod registration;
mod registry;
mod source;
#[cfg(test)]
mod tests;

pub use clock::{AnimationClock, SystemClock};
pub use entry::{ActiveAnimation, AnimationPlaybackState};
pub use handle::AnimationHandle;
pub use policy::MotionPolicy;
pub use registration::AnimationRegistration;
pub use registry::AnimationRegistry;
pub use source::AnimationSource;

use crate::{keyframes::Keyframes, timeline::Timeline, timing::Duration};

/// Runtime state owned by an Iced application.
#[derive(Debug, Clone)]
pub struct AnimationRuntime<C = SystemClock> {
    registry: AnimationRegistry,
    clock: C,
    motion_policy: MotionPolicy,
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

impl<C: AnimationClock> AnimationRuntime<C> {
    /// Creates an empty runtime with a custom clock.
    #[must_use]
    pub fn with_clock(clock: C) -> Self {
        Self {
            registry: AnimationRegistry::new(),
            clock,
            motion_policy: MotionPolicy::default(),
        }
    }

    /// Registers an animation source and returns its initial runtime output.
    pub fn register(&mut self, source: impl Into<AnimationSource>) -> AnimationRegistration {
        let handle = self.registry.allocate_handle();
        let started_at = self.clock.now();
        let source = source.into();
        let initial_snapshot = source.sample_at(Duration::ZERO);
        let mut entry = ActiveAnimation::new(handle, source, started_at);

        entry.set_last_snapshot(initial_snapshot.clone());

        if entry.source().total_duration() == Some(Duration::ZERO) {
            let completion_snapshot = entry.source().completion_snapshot();

            entry.set_last_snapshot(completion_snapshot);
            entry.mark_completed(started_at);
        }

        let registration =
            AnimationRegistration::from_entry(entry.handle(), entry.started_at(), &entry);

        self.registry.insert(entry);

        registration
    }

    /// Registers keyframes and returns their initial runtime output.
    pub fn register_keyframes(&mut self, keyframes: Keyframes) -> AnimationRegistration {
        self.register(keyframes)
    }

    /// Registers a timeline and returns its initial runtime output.
    pub fn register_timeline(&mut self, timeline: Timeline) -> AnimationRegistration {
        self.register(timeline)
    }
}

impl<C> AnimationRuntime<C> {
    /// Returns the active animation registry.
    #[must_use]
    pub const fn registry(&self) -> &AnimationRegistry {
        &self.registry
    }

    /// Returns mutable access to the active animation registry.
    #[must_use]
    pub const fn registry_mut(&mut self) -> &mut AnimationRegistry {
        &mut self.registry
    }

    /// Returns the runtime clock.
    #[must_use]
    pub const fn clock(&self) -> &C {
        &self.clock
    }

    /// Returns mutable access to the runtime clock.
    #[must_use]
    pub const fn clock_mut(&mut self) -> &mut C {
        &mut self.clock
    }

    /// Returns the current motion policy.
    #[must_use]
    pub const fn motion_policy(&self) -> MotionPolicy {
        self.motion_policy
    }

    /// Replaces the current motion policy.
    pub const fn set_motion_policy(&mut self, motion_policy: MotionPolicy) {
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
        self.registry.is_empty()
    }
}
