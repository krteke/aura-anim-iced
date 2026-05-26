use crate::property::{PropertySnapshot, sort_property_entries_by_composition};

/// A property snapshot stored at a normalized keyframe offset.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    offset: f32,
    snapshot: PropertySnapshot,
}

impl Keyframe {
    /// Creates a keyframe with a normalized offset and composition-sorted snapshot.
    #[must_use]
    pub fn new(offset: f32, mut snapshot: PropertySnapshot) -> Self {
        sort_property_entries_by_composition(&mut snapshot);

        Self {
            offset: normalize_offset(offset),
            snapshot,
        }
    }

    /// Returns the normalized offset of this keyframe.
    #[must_use]
    pub const fn offset(&self) -> f32 {
        self.offset
    }

    /// Returns a reference to the property snapshot at this keyframe.
    #[must_use]
    pub const fn snapshot(&self) -> &PropertySnapshot {
        &self.snapshot
    }

    pub(crate) fn normalize(&mut self) {
        self.offset = normalize_offset(self.offset);
        sort_property_entries_by_composition(&mut self.snapshot);
    }

    #[cfg(test)]
    pub(crate) fn new_unchecked(offset: f32, snapshot: PropertySnapshot) -> Self {
        Self { offset, snapshot }
    }
}

/// Normalizes a keyframe offset to the inclusive range `[0.0, 1.0]`.
#[must_use]
pub fn normalize_offset(offset: f32) -> f32 {
    if offset.is_finite() {
        offset.clamp(0.0, 1.0)
    } else {
        0.0
    }
}
