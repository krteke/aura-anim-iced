//! Timeline orchestration primitives.

mod duration;
mod error;
mod hold;
mod marker;
mod parallel;
mod sequence;
mod step;
#[cfg(test)]
mod tests;
mod track;

pub use error::TimelinePlaybackError;
pub use hold::Hold;
pub use marker::TimelineMarker;
pub use parallel::Parallel;
pub use sequence::Sequence;
pub use step::TimelineStep;
pub use track::{PropertyTrackBuilder, Track};

use crate::{keyframes::Keyframes, property::PropertySnapshot, timing::Duration};

/// A root timeline made of sequential steps.
///
/// # Example
///
/// ```
/// use aura_anim_iced::{
///     keyframes::KeyframesBuilder,
///     property,
///     timeline::{Hold, Timeline, Track},
///     timing::{Duration, Easing, Timing},
/// };
///
/// let enter = Track::new(
///     KeyframesBuilder::new()
///         .with_timing(Timing::new(120.0).with_easing(Easing::EaseOut))
///         .at(0.0, (property::OPACITY, 0.0))
///         .at(1.0, (property::OPACITY, 1.0))
///         .finish(),
/// );
/// let exit = Track::new(
///     KeyframesBuilder::new()
///         .with_timing(Timing::new(80.0).with_easing(Easing::EaseIn))
///         .at(0.0, (property::OPACITY, 1.0))
///         .at(1.0, (property::OPACITY, 0.0))
///         .finish(),
/// );
///
/// let timeline = Timeline::sequence([
///     enter.into(),
///     Hold::new(Duration::from_millis(400.0)).into(),
///     exit.into(),
/// ]);
///
/// assert_eq!(timeline.total_duration(), Some(Duration::from_millis(600.0)));
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timeline {
    name: Option<String>,
    root: Sequence,
    // markers: Vec<TimelineMarker>,
}

impl Timeline {
    /// Creates an empty timeline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the timeline name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Creates a timeline whose root sequence contains `steps`.
    #[must_use]
    pub fn sequence(steps: impl IntoIterator<Item = TimelineStep>) -> Self {
        Self {
            root: Sequence::from_steps(steps),
            ..Self::new()
        }
    }

    /// Creates a timeline with a single parallel group in the root sequence.
    #[must_use]
    pub fn parallel(steps: impl IntoIterator<Item = TimelineStep>) -> Self {
        Self::new().then(Parallel::from_steps(steps))
    }

    /// Creates a timeline with a single keyframe track in the root sequence.
    #[must_use]
    pub fn track(track: impl Into<Track>) -> Self {
        Self::new().then(track.into())
    }

    /// Creates a timeline with raw keyframes as a single track.
    #[must_use]
    pub fn keyframes(keyframes: Keyframes) -> Self {
        Self::track(keyframes)
    }

    /// Creates a timeline with a single hold segment.
    #[must_use]
    pub fn hold(duration: impl Into<Duration>) -> Self {
        Self::new().then(Hold::new(duration.into()))
    }

    /// Returns the timeline name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the root sequence.
    #[must_use]
    pub const fn root(&self) -> &Sequence {
        &self.root
    }

    // /// Returns the named markers sorted by offset.
    // #[must_use]
    // pub fn markers(&self) -> &[TimelineMarker] {
    //     &self.markers
    // }

    // /// Returns the first marker with `name`.
    // #[must_use]
    // pub fn marker_named(&self, name: &str) -> Option<&TimelineMarker> {
    //     self.markers.iter().find(|marker| marker.name() == name)
    // }

    // /// Returns markers whose offsets are at or before `offset`.
    // pub fn markers_at_or_before(
    //     &self,
    //     offset: impl Into<Duration>,
    // ) -> impl Iterator<Item = &TimelineMarker> + '_ {
    //     let offset = offset.into();

    //     self.markers
    //         .iter()
    //         .take_while(move |marker| marker.is_at_or_before(offset))
    // }

    /// Appends a timeline step to the root sequence.
    pub fn push_step(&mut self, step: impl Into<TimelineStep>) {
        self.root.push_step(step);
    }

    /// Appends a timeline step and returns the updated timeline.
    #[must_use]
    pub fn then(mut self, step: impl Into<TimelineStep>) -> Self {
        self.push_step(step);
        self
    }

    // /// Appends a named marker and returns the updated timeline.
    // #[must_use]
    // pub fn marker(mut self, name: impl Into<String>, offset: impl Into<Duration>) -> Self {
    //     self.push_marker(TimelineMarker::new(name, offset));
    //     self
    // }

    // /// Appends a named marker while preserving offset ordering.
    // pub fn push_marker(&mut self, marker: TimelineMarker) {
    //     let insert_at = self.markers.partition_point(|existing| {
    //         existing.offset().as_millis() <= marker.offset().as_millis()
    //     });

    //     self.markers.insert(insert_at, marker);
    // }

    /// Returns the finite total duration of the root sequence, or `None` if any step is infinite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        self.root.total_duration()
    }

    /// Samples the active root sequence step at `offset`.
    #[must_use]
    pub fn sample_at(&self, offset: impl Into<Duration>) -> Option<PropertySnapshot> {
        self.root.sample_at(offset.into())
    }

    /// Samples the final visual state of this timeline.
    #[must_use]
    pub fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        self.root.completion_snapshot()
    }
}
