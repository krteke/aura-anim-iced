//! Timeline orchestration primitives.

mod duration;
mod hold;
mod marker;
mod parallel;
mod sequence;
mod step;
#[cfg(test)]
mod tests;
mod track;

pub use hold::Hold;
pub use marker::TimelineMarker;
pub use parallel::Parallel;
pub use sequence::Sequence;
pub use step::TimelineStep;
pub use track::Track;

use crate::{property::PropertySnapshot, timing::Duration};

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
}
