use std::cmp::Ordering;

use super::{Keyframe, KeyframeSegment};
use crate::{property::PropertySnapshot, timing::Timing};

/// A collection of property snapshots keyed by normalized offsets.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Keyframes {
    frames: Vec<Keyframe>,
    timing: Timing,
}

impl Keyframes {
    /// Creates an empty keyframe track with default timing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the keyframes in this track.
    #[must_use]
    pub fn frames(&self) -> &[Keyframe] {
        &self.frames
    }

    /// Returns the timing attached to this keyframe track.
    #[must_use]
    pub const fn timing(&self) -> &Timing {
        &self.timing
    }

    /// Returns `true` if this keyframe track is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Returns the number of keyframes in this track.
    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Sets the timing attached to this keyframe track.
    #[must_use]
    pub const fn with_timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }

    /// Inserts a property snapshot at a normalized offset and returns the updated track.
    #[must_use]
    pub fn at(mut self, offset: f32, snapshot: impl Into<PropertySnapshot>) -> Self {
        self.push_at(offset, snapshot);
        self
    }

    /// Inserts a property snapshot at a normalized offset.
    pub fn push_at(&mut self, offset: f32, snapshot: impl Into<PropertySnapshot>) {
        self.frames.push(Keyframe::new(offset, snapshot.into()));
        self.sort_frames();
    }

    /// Normalizes all keyframe offsets, snapshot property ordering, and frame ordering.
    pub fn normalize(&mut self) {
        self.deep_normalize();
    }

    /// Finds the keyframe segment that contains a normalized offset.
    #[must_use]
    pub fn segment_at(&self, offset: f32) -> KeyframeSegment<'_> {
        KeyframeSegment::find(&self.frames, offset)
    }

    pub(crate) fn deep_normalize(&mut self) {
        for frame in &mut self.frames {
            frame.normalize();
        }

        self.sort_frames();
    }

    fn sort_frames(&mut self) {
        self.frames.sort_by(|left, right| {
            left.offset()
                .partial_cmp(&right.offset())
                .unwrap_or(Ordering::Equal)
        });
    }

    #[cfg(test)]
    pub(crate) fn from_raw_frames(frames: Vec<Keyframe>, timing: Timing) -> Self {
        Self { frames, timing }
    }
}
