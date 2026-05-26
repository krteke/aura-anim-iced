use super::{AnimationHandle, AnimationSource};
use crate::{property::PropertySnapshot, timing::Duration};

/// Playback state tracked for an active runtime entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnimationPlaybackState {
    /// The animation can advance on runtime ticks.
    #[default]
    Playing,
    /// The animation is held at its current elapsed position.
    Paused,
    /// The animation has been canceled and should stop producing snapshots.
    Canceled,
    /// The animation reached completion.
    Completed,
}

/// A runtime registry entry for one animation source.
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveAnimation {
    handle: AnimationHandle,
    source: AnimationSource,
    started_at: Duration,
    state: AnimationPlaybackState,
    last_snapshot: Option<PropertySnapshot>,
}

impl ActiveAnimation {
    /// Creates an active animation entry.
    #[must_use]
    pub fn new(
        handle: AnimationHandle,
        source: impl Into<AnimationSource>,
        started_at: impl Into<Duration>,
    ) -> Self {
        Self {
            handle,
            source: source.into(),
            started_at: started_at.into(),
            state: AnimationPlaybackState::Playing,
            last_snapshot: None,
        }
    }

    /// Returns the handle for this entry.
    #[must_use]
    pub const fn handle(&self) -> AnimationHandle {
        self.handle
    }

    /// Returns the animation source.
    #[must_use]
    pub const fn source(&self) -> &AnimationSource {
        &self.source
    }

    /// Returns the runtime timestamp when this entry started.
    #[must_use]
    pub const fn started_at(&self) -> Duration {
        self.started_at
    }

    /// Returns the current playback state.
    #[must_use]
    pub const fn state(&self) -> AnimationPlaybackState {
        self.state
    }

    /// Sets the current playback state.
    pub const fn set_state(&mut self, state: AnimationPlaybackState) {
        self.state = state;
    }

    /// Returns the last sampled property snapshot.
    #[must_use]
    pub const fn last_snapshot(&self) -> Option<&PropertySnapshot> {
        self.last_snapshot.as_ref()
    }

    /// Stores the last sampled property snapshot.
    pub fn set_last_snapshot(&mut self, snapshot: Option<PropertySnapshot>) {
        self.last_snapshot = snapshot;
    }
}
