//! Runtime animation lifecycle events and stable identifiers.

use super::Motion;

/// Identifies one runtime motion slot lifecycle.
///
/// A slot's generation changes when storage is reused, so events from a
/// removed motion never match a later motion occupying the same slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MotionId {
    slot: usize,
    generation: u64,
}

impl MotionId {
    pub(super) const fn new(slot: usize, generation: u64) -> Self {
        Self { slot, generation }
    }

    /// Returns the runtime slot index.
    #[must_use]
    pub const fn slot(self) -> usize {
        self.slot
    }

    /// Returns the slot generation captured by this ID.
    #[must_use]
    pub const fn generation(self) -> u64 {
        self.generation
    }
}

/// Identifies one concrete playback on a motion.
///
/// Replacing or retargeting a motion creates a new playback ID even though the
/// typed [`Motion`] handle remains unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaybackId {
    motion: MotionId,
    revision: u64,
}

impl PlaybackId {
    pub(super) const fn new(motion: MotionId, revision: u64) -> Self {
        Self { motion, revision }
    }

    /// Returns the motion owning this playback.
    #[must_use]
    pub const fn motion(self) -> MotionId {
        self.motion
    }

    /// Returns the playback revision within the motion lifecycle.
    #[must_use]
    pub const fn revision(self) -> u64 {
        self.revision
    }
}

/// Explains why an unfinished playback stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InterruptionReason {
    /// A new animation replaced the playback.
    Replaced,
    /// A transition retargeted the playback.
    Retargeted,
    /// The owning motion was removed.
    Removed,
}

/// Explains why a motion left runtime storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RemovalReason {
    /// The application explicitly removed the motion.
    Explicit,
    /// The motion used [`super::RetainPolicy::DropWhenSettled`].
    Settled,
}

/// The lifecycle change represented by a [`MotionEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MotionEventKind {
    /// The playback reached its completed state.
    Completed,
    /// The playback was canceled.
    Canceled,
    /// The playback stopped because another operation superseded it.
    Interrupted(InterruptionReason),
    /// The owning motion was removed from runtime storage.
    Removed(RemovalReason),
}

/// A structured runtime lifecycle event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MotionEvent {
    motion: MotionId,
    playback: PlaybackId,
    kind: MotionEventKind,
}

impl MotionEvent {
    pub(super) const fn new(playback: PlaybackId, kind: MotionEventKind) -> Self {
        Self {
            motion: playback.motion(),
            playback,
            kind,
        }
    }

    /// Returns the motion associated with this event.
    #[must_use]
    pub const fn motion(self) -> MotionId {
        self.motion
    }

    /// Returns the concrete playback associated with this event.
    #[must_use]
    pub const fn playback(self) -> PlaybackId {
        self.playback
    }

    /// Returns the event kind.
    #[must_use]
    pub const fn kind(self) -> MotionEventKind {
        self.kind
    }

    /// Returns whether this event belongs to `target`.
    #[must_use]
    pub fn is_for(self, target: impl MotionEventTarget) -> bool {
        target.matches(self)
    }

    /// Returns whether this is a completed event for `target`.
    #[must_use]
    pub fn is_completed_for(self, target: impl MotionEventTarget) -> bool {
        self.kind == MotionEventKind::Completed && target.matches(self)
    }

    /// Returns whether this is a canceled event for `target`.
    #[must_use]
    pub fn is_canceled_for(self, target: impl MotionEventTarget) -> bool {
        self.kind == MotionEventKind::Canceled && target.matches(self)
    }
}

mod private {
    pub trait Sealed {}

    impl<T> Sealed for super::Motion<T> {}
    impl Sealed for super::MotionId {}
    impl Sealed for super::PlaybackId {}
}

/// A motion or playback that can be matched against a [`MotionEvent`].
pub trait MotionEventTarget: private::Sealed {
    /// Returns whether this target matches `event`.
    #[doc(hidden)]
    fn matches(self, event: MotionEvent) -> bool;
}

impl<T> MotionEventTarget for Motion<T> {
    fn matches(self, event: MotionEvent) -> bool {
        self.id() == event.motion
    }
}

impl MotionEventTarget for MotionId {
    fn matches(self, event: MotionEvent) -> bool {
        self == event.motion
    }
}

impl MotionEventTarget for PlaybackId {
    fn matches(self, event: MotionEvent) -> bool {
        self == event.playback
    }
}
