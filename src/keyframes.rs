//! Property-snapshot keyframe storage.

use std::cmp::Ordering;

use crate::{
    property::{PropertySnapshot, sort_property_entries_by_composition},
    timing::Timing,
};

/// A collection of property snapshots keyed by normalized offsets.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Keyframes {
    /// Sorted keyframes.
    frames: Vec<Keyframe>,
    /// Timing attached to this keyframe track.
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
        self.sort_frames();
    }

    fn deep_normalize(&mut self) {
        for frame in &mut self.frames {
            frame.normalize();
        }

        self.sort_frames();
    }

    fn sort_frames(&mut self) {
        self.frames.sort_by(|left, right| {
            left.offset
                .partial_cmp(&right.offset)
                .unwrap_or(Ordering::Equal)
        });
    }
}

/// A property snapshot stored at a normalized keyframe offset.
#[derive(Debug, Clone, PartialEq)]
pub struct Keyframe {
    /// Normalized offset in the inclusive range `[0.0, 1.0]`.
    offset: f32,
    /// Property values captured at this offset.
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

    fn normalize(&mut self) {
        self.offset = normalize_offset(self.offset);
        sort_property_entries_by_composition(&mut self.snapshot);
    }
}

fn normalize_offset(offset: f32) -> f32 {
    if offset.is_finite() {
        offset.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Keyframe, Keyframes};
    use crate::{
        property::{PropertyValue, UiProperty},
        timing::{Delay, Timing},
    };

    fn snapshot(entries: &[(UiProperty, f32)]) -> Vec<(UiProperty, PropertyValue)> {
        entries
            .iter()
            .map(|(property, value)| (*property, PropertyValue::Scalar(*value)))
            .collect()
    }

    #[test]
    fn new_keyframes_start_empty_with_default_timing() {
        let keyframes = Keyframes::new();

        assert!(keyframes.frames.is_empty());
        assert_eq!(keyframes.timing, Timing::default());
    }

    #[test]
    fn with_timing_attaches_timing_to_track() {
        let timing = Timing::new(250.0).with_delay(Delay::from_millis(50.0));

        let keyframes = Keyframes::new().with_timing(timing);

        assert_eq!(keyframes.timing, timing);
    }

    #[test]
    fn at_inserts_keyframes_in_sorted_offset_order() {
        let keyframes = Keyframes::new()
            .at(0.75, snapshot(&[(UiProperty::Opacity, 0.75)]))
            .at(0.25, snapshot(&[(UiProperty::Opacity, 0.25)]))
            .at(0.5, snapshot(&[(UiProperty::Opacity, 0.5)]));

        let offsets: Vec<_> = keyframes.frames.iter().map(|frame| frame.offset).collect();

        assert_eq!(offsets, vec![0.25, 0.5, 0.75]);
    }

    #[test]
    fn offsets_are_clamped_and_invalid_offsets_become_zero() {
        let keyframes = Keyframes::new()
            .at(1.25, snapshot(&[(UiProperty::Opacity, 1.0)]))
            .at(-0.5, snapshot(&[(UiProperty::Opacity, 0.0)]))
            .at(f32::NAN, snapshot(&[(UiProperty::Opacity, 0.5)]));

        let offsets: Vec<_> = keyframes.frames.iter().map(|frame| frame.offset).collect();

        assert_eq!(offsets, vec![0.0, 0.0, 1.0]);
    }

    #[test]
    fn keyframe_snapshots_are_sorted_by_property_composition() {
        let frame = Keyframe::new(
            0.5,
            snapshot(&[
                (UiProperty::Shadow, 1.0),
                (UiProperty::Opacity, 0.5),
                (UiProperty::Radius, 8.0),
                (UiProperty::TranslateX, 12.0),
            ]),
        );

        let properties: Vec<_> = frame
            .snapshot
            .iter()
            .map(|(property, _)| *property)
            .collect();

        assert_eq!(
            properties,
            vec![
                UiProperty::Opacity,
                UiProperty::TranslateX,
                UiProperty::Radius,
                UiProperty::Shadow,
            ]
        );
    }

    #[test]
    fn normalize_repairs_manually_mutated_frames() {
        let mut keyframes = Keyframes {
            frames: vec![
                Keyframe {
                    offset: 2.0,
                    snapshot: snapshot(&[(UiProperty::Shadow, 1.0), (UiProperty::Opacity, 0.0)]),
                },
                Keyframe {
                    offset: -1.0,
                    snapshot: snapshot(&[(UiProperty::Radius, 4.0)]),
                },
            ],
            timing: Timing::default(),
        };

        keyframes.deep_normalize();

        assert_eq!(keyframes.frames[0].offset, 0.0);
        assert_eq!(keyframes.frames[1].offset, 1.0);
        assert_eq!(keyframes.frames[1].snapshot[0].0, UiProperty::Opacity);
    }
}
