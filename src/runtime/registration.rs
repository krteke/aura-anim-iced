use super::{ActiveAnimation, AnimationHandle, AnimationPlaybackState};
use crate::{property::PropertySnapshot, timing::Duration};

/// Output produced when an animation is registered with the runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationRegistration {
    handle: AnimationHandle,
    started_at: Duration,
    state: AnimationPlaybackState,
    properties: Option<PropertySnapshot>,
    completed_at: Option<Duration>,
}

impl AnimationRegistration {
    pub(super) fn from_entry(
        handle: AnimationHandle,
        started_at: Duration,
        entry: &ActiveAnimation,
    ) -> Self {
        Self {
            handle,
            started_at,
            state: entry.state(),
            properties: entry.last_snapshot().cloned(),
            completed_at: entry.completed_at(),
        }
    }

    /// Returns the registered animation handle.
    #[must_use]
    pub const fn handle(&self) -> AnimationHandle {
        self.handle
    }

    /// Returns the runtime timestamp used as the animation start time.
    #[must_use]
    pub const fn started_at(&self) -> Duration {
        self.started_at
    }

    /// Returns the initial playback state.
    #[must_use]
    pub const fn state(&self) -> AnimationPlaybackState {
        self.state
    }

    /// Returns the initial property snapshot produced during registration.
    #[must_use]
    pub const fn properties(&self) -> Option<&PropertySnapshot> {
        self.properties.as_ref()
    }

    /// Returns the completion timestamp for animations completed at registration.
    #[must_use]
    pub const fn completed_at(&self) -> Option<Duration> {
        self.completed_at
    }
}
