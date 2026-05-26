use super::{Hold, Parallel, Sequence, Track};
use crate::{property::PropertySnapshot, timing::Duration};

/// A timeline step that can contribute duration to a timeline.
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineStep {
    /// A keyframe track.
    Track(Track),
    /// A sequential group of steps.
    Sequence(Sequence),
    /// A parallel group of steps.
    Parallel(Parallel),
    /// A silent hold segment.
    Hold(Hold),
}

impl TimelineStep {
    /// Returns the finite total duration of this step, or `None` for infinite tracks.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        match self {
            Self::Track(track) => track.total_duration(),
            Self::Sequence(sequence) => sequence.total_duration(),
            Self::Parallel(parallel) => parallel.total_duration(),
            Self::Hold(hold) => Some(hold.total_duration()),
        }
    }

    /// Samples this step at local timeline `offset`.
    #[must_use]
    pub fn sample_at(&self, offset: impl Into<Duration>) -> Option<PropertySnapshot> {
        match self {
            Self::Track(track) => track.sample_at(offset),
            Self::Sequence(sequence) => sequence.sample_at(offset),
            Self::Parallel(parallel) => parallel.sample_at(offset),
            Self::Hold(_) => None,
        }
    }

    /// Samples the final visual state for this step.
    #[must_use]
    pub fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        match self {
            Self::Track(track) => track.completion_snapshot(),
            Self::Sequence(sequence) => sequence.completion_snapshot(),
            Self::Parallel(parallel) => parallel.completion_snapshot(),
            Self::Hold(_) => None,
        }
    }
}

impl From<Track> for TimelineStep {
    fn from(value: Track) -> Self {
        Self::Track(value)
    }
}

impl From<Sequence> for TimelineStep {
    fn from(value: Sequence) -> Self {
        Self::Sequence(value)
    }
}

impl From<Parallel> for TimelineStep {
    fn from(value: Parallel) -> Self {
        Self::Parallel(value)
    }
}

impl From<Hold> for TimelineStep {
    fn from(value: Hold) -> Self {
        Self::Hold(value)
    }
}
