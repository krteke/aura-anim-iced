//! Property-snapshot keyframe storage and lookup.

mod frame;
mod sample;
#[cfg(test)]
mod segment;
#[cfg(test)]
mod tests;
mod track;

use std::collections::HashMap;

pub use frame::{Keyframe, normalize_offset};

use crate::{
    PropertySnapshot, Timing,
    keyframes::{
        sample::{sample_frames, sample_frames_into},
        track::PropertyTrack,
    },
    nearly_equal_f32, property,
};

/// A collection of property snapshots keyed by normalized offsets.
///
/// `Keyframes` is the compact, sampled representation used by timelines and
/// the runtime. Build one with [`KeyframesBuilder`].
///
/// # Example
///
/// ```
/// use aura_anim_iced::{Easing, KeyframesBuilder, Timing, property};
///
/// let keyframes = KeyframesBuilder::new()
///     .with_timing(Timing::new(180.0).with_easing(Easing::EaseOut))
///     .at(0.0, (property::OPACITY, 0.0))
///     .at(0.0, (property::SCALE, 0.95))
///     .at(1.0, (property::OPACITY, 1.0))
///     .at(1.0, (property::SCALE, 1.0))
///     .finish();
///
/// let sample = keyframes.sample_at(0.5).expect("active sample");
///
/// assert!(sample.find_property(&property::OPACITY.raw()).is_some());
/// assert!(sample.find_property(&property::SCALE.raw()).is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Keyframes {
    tracks: Vec<PropertyTrack>,
    timing: Timing,
}

impl Keyframes {
    /// Creates a keyframe builder.
    #[must_use]
    pub fn builder() -> KeyframesBuilder {
        KeyframesBuilder::new()
    }

    /// Returns `true` when this keyframe set has no property tracks.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// Returns the number of property tracks in this keyframe set.
    #[must_use]
    pub fn track_count(&self) -> usize {
        self.tracks.len()
    }

    /// Returns the timing attached to this keyframe set.
    #[must_use]
    pub fn timing(&self) -> &Timing {
        &self.timing
    }

    /// Samples all property tracks at a normalized offset.
    #[must_use]
    pub fn sample_at(&self, offset: f32) -> Option<PropertySnapshot> {
        sample_frames(&self.tracks, offset, self.timing.easing())
    }

    pub(crate) fn sample_into(&self, offset: f32, output: &mut PropertySnapshot) -> bool {
        sample_frames_into(&self.tracks, offset, self.timing.easing(), output)
    }

    pub(crate) fn sample_completion(&self) -> Option<PropertySnapshot> {
        let iteration_count = self.timing.iterations().finite_count()?;
        let offset = self.timing.direction().end_progress(iteration_count);

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
        )]
        self.sample_at(offset as f32)
    }

    pub(crate) fn sample_completion_into(&self, output: &mut PropertySnapshot) -> bool {
        let Some(iteration_count) = self.timing.iterations().finite_count() else {
            output.clear();
            return false;
        };
        let offset = self.timing.direction().end_progress(iteration_count);

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
        )]
        self.sample_into(offset as f32, output)
    }
}

/// Builder that collects offset-keyed snapshots before compiling them into
/// per-property tracks.
///
/// Duplicate offsets are merged as snapshots are inserted. If the same property
/// appears more than once at an offset, the later value wins.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KeyframesBuilder {
    frames: Vec<Keyframe>,
    timing: Timing,
}

impl KeyframesBuilder {
    /// Creates an empty keyframe builder with default timing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the timing that will be attached to the finished keyframes.
    #[must_use]
    pub const fn timing(&self) -> &Timing {
        &self.timing
    }

    /// Sets the timing attached to the finished keyframes.
    #[must_use]
    pub const fn with_timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }

    /// Inserts a property snapshot at a normalized offset.
    #[must_use]
    pub fn at(mut self, offset: f32, snapshot: impl Into<PropertySnapshot>) -> Self {
        self.push_at(offset, snapshot);
        self
    }

    /// Inserts an opacity keyframe.
    #[must_use]
    pub fn opacity(self, offset: f32, value: f32) -> Self {
        self.at(offset, (property::OPACITY, value))
    }

    /// Inserts a uniform scale keyframe.
    #[must_use]
    pub fn scale(self, offset: f32, value: f32) -> Self {
        self.at(offset, (property::SCALE, value))
    }

    /// Inserts a translation keyframe.
    #[must_use]
    pub fn translation(self, offset: f32, x: f32, y: f32) -> Self {
        self.at(offset, (property::TRANSLATE, iced::Vector::new(x, y)))
    }

    /// Inserts a background color keyframe.
    #[must_use]
    pub fn background_color(self, offset: f32, value: iced::Color) -> Self {
        self.at(offset, (property::BACKGROUND, value))
    }

    /// Inserts a border color keyframe.
    #[must_use]
    pub fn border_color(self, offset: f32, value: iced::Color) -> Self {
        self.at(offset, (property::BORDER_COLOR, value))
    }

    /// Inserts a text color keyframe.
    #[must_use]
    pub fn text_color(self, offset: f32, value: iced::Color) -> Self {
        self.at(offset, (property::TEXT_COLOR, value))
    }

    /// Inserts a shadow keyframe.
    #[must_use]
    pub fn shadow(self, offset: f32, value: iced::Shadow) -> Self {
        self.at(offset, (property::SHADOW, value))
    }

    /// Inserts a property snapshot at a normalized offset.
    pub fn push_at(&mut self, offset: f32, snapshot: impl Into<PropertySnapshot>) {
        self.upsert_frame(Keyframe::new(offset, snapshot.into()));
    }

    /// Inserts multiple property snapshots and normalizes them in one pass.
    pub fn push_many<I, S>(&mut self, frames: I)
    where
        I: IntoIterator<Item = (f32, S)>,
        S: Into<PropertySnapshot>,
    {
        self.frames.extend(
            frames
                .into_iter()
                .map(|(offset, snapshot)| Keyframe::new(offset, snapshot.into())),
        );
        self.sort_and_merge_frames();
    }

    fn upsert_frame(&mut self, frame: Keyframe) {
        if let Some(existing) = self
            .frames
            .iter_mut()
            .find(|existing| nearly_equal_f32(existing.offset(), frame.offset()))
        {
            existing.merge_snapshot(frame.snapshot().clone());
            return;
        }

        let insert_at = self
            .frames
            .partition_point(|existing| existing.offset() < frame.offset());
        self.frames.insert(insert_at, frame);
    }

    fn sort_and_merge_frames(&mut self) {
        self.frames
            .sort_by(|left, right| left.offset().total_cmp(&right.offset()));

        let mut merged = Vec::with_capacity(self.frames.len());

        for frame in self.frames.drain(..) {
            if let Some(existing) = merged.last_mut().filter(|existing: &&mut Keyframe| {
                nearly_equal_f32(existing.offset(), frame.offset())
            }) {
                existing.merge_snapshot(frame.snapshot().clone());
            } else {
                merged.push(frame);
            }
        }

        self.frames = merged;
    }

    /// Compiles the collected snapshots into per-property tracks.
    #[must_use]
    pub fn finish(self) -> Keyframes {
        let mut track_map = HashMap::new();

        for frame in &self.frames {
            for entry in frame.snapshot().entries() {
                let track = track_map
                    .entry(*entry.spec())
                    .or_insert_with(|| PropertyTrack::new(*entry.spec(), Vec::new()));
                track.add_sample(frame.offset(), *entry.value());
            }
        }

        let mut tracks: Vec<PropertyTrack> = track_map
            .into_values()
            .map(|mut track| {
                track.sort_samples();
                track
            })
            .collect();
        tracks.sort_by_key(|track| track.spec().composition_order());

        Keyframes {
            tracks,
            timing: self.timing,
        }
    }
}
