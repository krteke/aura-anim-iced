//! Timeline orchestration primitives.

mod duration;
mod error;
mod hold;
mod marker;
mod parallel;
mod playback;
mod sequence;
mod step;
#[cfg(test)]
mod tests;
mod track;

pub use hold::Hold;
pub use marker::TimelineMarker;
pub use parallel::Parallel;
pub use playback::{TimelinePlayback, TimelinePlaybackSnapshot, TimelinePlaybackState};
pub use sequence::Sequence;
pub use step::TimelineStep;
pub use track::Track;

use crate::{keyframes::Keyframes, property::PropertySnapshot, timing::Duration};

/// A root timeline made of sequential steps.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timeline {
    name: Option<String>,
    root: Sequence,
    markers: Vec<TimelineMarker>,
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

    /// Returns the named markers in insertion order.
    #[must_use]
    pub fn markers(&self) -> &[TimelineMarker] {
        &self.markers
    }

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

    /// Appends a named marker.
    pub fn push_marker(&mut self, marker: TimelineMarker) {
        self.markers.push(marker);
    }

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
