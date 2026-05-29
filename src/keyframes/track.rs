use super::{Keyframe, KeyframeSegment, sample::sample_frames};
use crate::{
    nearly_equal_f32,
    property::{self, PropertySnapshot},
    timing::Timing,
};

/// A collection of property snapshots keyed by normalized offsets.
///
/// # Example
///
/// ```
/// use aura_anim_iced::{Easing, Keyframes, Timing, property};
///
/// let keyframes = Keyframes::new()
///     .with_timing(Timing::new(180.0).with_easing(Easing::EaseOut))
///     .at(0.0, (property::OPACITY, 0.0))
///     .at(0.0, (property::SCALE, 0.95))
///     .at(1.0, (property::OPACITY, 1.0))
///     .at(1.0, (property::SCALE, 1.0));
///
/// let sample = keyframes.sample_at(0.5).expect("active sample");
///
/// assert!(sample.find_property(&property::OPACITY.raw()).is_some());
/// assert!(sample.find_property(&property::SCALE.raw()).is_some());
/// ```
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

    /// Inserts horizontal and vertical translation keyframes at the same offset.
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

    /// Inserts multiple property snapshots and returns the updated track.
    #[must_use]
    pub fn with_keyframes<I, S>(mut self, frames: I) -> Self
    where
        I: IntoIterator<Item = (f32, S)>,
        S: Into<PropertySnapshot>,
    {
        self.push_many(frames);
        self
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

    /// Normalizes all keyframe offsets, snapshot property ordering, and frame ordering.
    pub fn normalize(&mut self) {
        self.sort_and_merge_frames();
    }

    /// Finds the keyframe segment that contains a normalized offset.
    #[must_use]
    pub fn segment_at(&self, offset: f32) -> KeyframeSegment<'_> {
        KeyframeSegment::find(&self.frames, offset)
    }

    /// Samples a property snapshot at a normalized offset.
    #[must_use]
    pub fn sample_at(&self, offset: f32) -> Option<PropertySnapshot> {
        sample_frames(&self.frames, offset, self.timing.easing())
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
}
