use super::AnimationHandle;
use crate::{
    property::PropertySnapshot, runtime::source::AnimationSource,
    runtime::target::AnimationTargetId, timing::Duration,
};

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

/// Internal runtime registry entry for one animation source.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ActiveAnimation {
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
    #[must_use]
    pub(crate) fn new(
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

    #[must_use]
    pub(crate) const fn handle(&self) -> AnimationHandle {
        self.handle
    }

    #[must_use]
    pub(crate) const fn source(&self) -> &AnimationSource {
        &self.source
    }

    #[must_use]
    pub(crate) const fn target(&self) -> AnimationTargetId {
        self.target
    }

    #[must_use]
    pub(crate) const fn last_tick(&self) -> Duration {
        self.last_tick
    }

    #[must_use]
    pub(crate) const fn position(&self) -> Duration {
        self.position
    }

    #[must_use]
    pub(crate) const fn state(&self) -> AnimationPlaybackState {
        self.state
    }

    pub(crate) const fn set_state(&mut self, state: AnimationPlaybackState) {
        self.state = state;
    }

    #[must_use]
    pub(crate) const fn last_snapshot(&self) -> Option<&PropertySnapshot> {
        self.last_snapshot.as_ref()
    }

    pub(crate) fn set_last_snapshot(&mut self, snapshot: Option<PropertySnapshot>) {
        self.last_snapshot = snapshot;
    }

    #[must_use]
    pub(crate) const fn is_active(&self) -> bool {
        matches!(
            self.state,
            AnimationPlaybackState::Playing | AnimationPlaybackState::Paused
        )
    }

    #[must_use]
    pub(crate) const fn needs_tick(&self) -> bool {
        matches!(self.state, AnimationPlaybackState::Playing)
    }

    #[must_use]
    pub(crate) const fn completed_at(&self) -> Option<Duration> {
        self.completed_at
    }

    pub(crate) const fn mark_completed(&mut self, completed_at: Duration) {
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
