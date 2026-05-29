use super::{AnimationHandle, AnimationSource};
use crate::{property::PropertySnapshot, runtime::target::AnimationTargetId, timing::Duration};

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
    target: AnimationTargetId,
    source: AnimationSource,
    position: Duration,
    last_tick: Duration,
    state: AnimationPlaybackState,
    last_snapshot: Option<PropertySnapshot>,
    completed_at: Option<Duration>,
}

impl ActiveAnimation {
    /// Creates an active animation entry.
    #[must_use]
    pub fn new(
        handle: AnimationHandle,
        target: AnimationTargetId,
        source: impl Into<AnimationSource>,
        now: impl Into<Duration>,
    ) -> Self {
        Self {
            handle,
            target,
            source: source.into(),
            position: Duration::ZERO,
            last_tick: now.into(),
            state: AnimationPlaybackState::Playing,
            last_snapshot: None,
            completed_at: None,
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

    /// Returns the target of this animation.
    #[must_use]
    pub const fn target(&self) -> AnimationTargetId {
        self.target
    }

    /// Returns the runtime timestamp of the last tick.
    #[must_use]
    pub const fn last_tick(&self) -> Duration {
        self.last_tick
    }

    /// Returns the current playback position.
    #[must_use]
    pub const fn position(&self) -> Duration {
        self.position
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

    /// Returns whether this entry has completed.
    #[must_use]
    pub const fn is_completed(&self) -> bool {
        self.completed_at.is_some()
    }

    /// Returns whether this entry is still active runtime state.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(
            self.state,
            AnimationPlaybackState::Playing | AnimationPlaybackState::Paused
        )
    }

    /// Returns whether this entry needs runtime tick updates.
    #[must_use]
    pub const fn needs_tick(&self) -> bool {
        matches!(self.state, AnimationPlaybackState::Playing)
    }

    /// Returns the runtime timestamp when this entry completed.
    #[must_use]
    pub const fn completed_at(&self) -> Option<Duration> {
        self.completed_at
    }

    /// Marks this entry as completed.
    pub const fn mark_completed(&mut self, completed_at: Duration) {
        self.state = AnimationPlaybackState::Completed;
        self.completed_at = Some(completed_at);
    }

    pub(super) fn update_position(&mut self, delta: Duration) {
        self.position += delta;
    }

    pub(super) fn set_last_tick(&mut self, last_tick: Duration) {
        self.last_tick = last_tick;
    }

    pub(super) fn set_position(&mut self, position: Duration) {
        self.position = position;
    }
}
