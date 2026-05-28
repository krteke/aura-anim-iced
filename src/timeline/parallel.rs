use super::{Hold, Sequence, TimelineStep, Track, duration::max_duration};
use crate::{keyframes::Keyframes, property::PropertySnapshot, timing::Duration};

/// A parallel timeline group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Parallel {
    steps: Vec<TimelineStep>,
}

impl Parallel {
    /// Creates an empty parallel group.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a parallel group from steps.
    #[must_use]
    pub fn from_steps(steps: impl IntoIterator<Item = TimelineStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }

    /// Returns the parallel steps in insertion order.
    #[must_use]
    pub fn steps(&self) -> &[TimelineStep] {
        &self.steps
    }

    /// Appends a timeline step.
    pub fn push_step(&mut self, step: impl Into<TimelineStep>) {
        self.steps.push(step.into());
    }

    /// Appends a timeline step and returns the updated parallel group.
    #[must_use]
    pub fn then(mut self, step: impl Into<TimelineStep>) -> Self {
        self.push_step(step);
        self
    }

    /// Appends a keyframe track.
    #[must_use]
    pub fn track(self, track: impl Into<Track>) -> Self {
        self.then(track.into())
    }

    /// Appends raw keyframes as a track.
    #[must_use]
    pub fn keyframes(self, keyframes: Keyframes) -> Self {
        self.track(keyframes)
    }

    /// Appends a hold segment.
    #[must_use]
    pub fn hold(self, duration: impl Into<Duration>) -> Self {
        self.then(Hold::new(duration.into()))
    }

    /// Appends a sequence.
    #[must_use]
    pub fn sequence(self, sequence: Sequence) -> Self {
        self.then(sequence)
    }

    /// Appends a nested parallel group.
    #[must_use]
    pub fn parallel_step(self, parallel: Parallel) -> Self {
        self.then(parallel)
    }

    /// Returns the finite maximum step duration, or `None` if any step is infinite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        max_duration(self.steps.iter().map(TimelineStep::total_duration))
    }

    /// Samples this parallel group at local timeline `offset`.
    #[must_use]
    pub fn sample_at(&self, offset: impl Into<Duration>) -> Option<PropertySnapshot> {
        let offset = offset.into();
        let mut merged = PropertySnapshot::new();

        for step in &self.steps {
            if let Some(snapshot) = step.sample_at(offset) {
                merged.merge(snapshot);
            }
        }

        if merged.is_empty() {
            None
        } else {
            merged.sort_by_composition_key();
            Some(merged)
        }
    }

    /// Samples the final visual state of all child steps.
    #[must_use]
    pub fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        let mut merged = PropertySnapshot::new();

        for step in &self.steps {
            if let Some(snapshot) = step.completion_snapshot() {
                merged.merge(snapshot);
            }
        }

        if merged.is_empty() {
            None
        } else {
            merged.sort_by_composition_key();
            Some(merged)
        }
    }
}
