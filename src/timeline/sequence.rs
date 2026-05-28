use super::{
    Hold, Parallel, TimelineStep, Track,
    duration::{contains_offset, sum_durations},
};
use crate::{keyframes::Keyframes, property::PropertySnapshot, timing::Duration};

/// A sequential timeline group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sequence {
    steps: Vec<TimelineStep>,
}

impl Sequence {
    /// Creates an empty sequence.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sequence from steps.
    #[must_use]
    pub fn from_steps(steps: impl IntoIterator<Item = TimelineStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }

    /// Returns the steps in insertion order.
    #[must_use]
    pub fn steps(&self) -> &[TimelineStep] {
        &self.steps
    }

    /// Appends a timeline step.
    pub fn push_step(&mut self, step: impl Into<TimelineStep>) {
        self.steps.push(step.into());
    }

    /// Appends a timeline step and returns the updated sequence.
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

    /// Appends a nested sequence.
    #[must_use]
    pub fn sequence_step(self, sequence: Sequence) -> Self {
        self.then(sequence)
    }

    /// Appends a parallel group.
    #[must_use]
    pub fn parallel(self, parallel: Parallel) -> Self {
        self.then(parallel)
    }

    /// Returns the finite sum of all step durations, or `None` if any step is infinite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        sum_durations(self.steps.iter().map(TimelineStep::total_duration))
    }

    /// Samples the active ordered step at `offset`.
    #[must_use]
    pub fn sample_at(&self, offset: impl Into<Duration>) -> Option<PropertySnapshot> {
        let offset_dur = offset.into();
        let mut cursor_dur = Duration::ZERO;
        let last_index = self.steps.len().saturating_sub(1);
        let mut last_visible_snapshot = None;

        for (index, step) in self.steps.iter().enumerate() {
            let Some(duration) = step.total_duration() else {
                return step.sample_at(offset_dur.checked_sub(cursor_dur)?);
            };
            let end_dur = cursor_dur + duration;

            if contains_offset(
                cursor_dur.as_millis(),
                end_dur.as_millis(),
                offset_dur.as_millis(),
                index == last_index,
            ) {
                if step.is_hold() {
                    return last_visible_snapshot;
                }
                return step.sample_at(offset_dur.checked_sub(cursor_dur)?);
            }

            if let Some(snapshot) = step.completion_snapshot() {
                last_visible_snapshot = Some(snapshot);
            }

            cursor_dur = end_dur;
        }

        None
    }

    /// Samples the final visual state from the last step that can produce one.
    #[must_use]
    pub fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        let mut snapshot = PropertySnapshot::new();
        let snapshots = self
            .steps
            .iter()
            .map(|s| s.completion_snapshot())
            .collect::<Vec<_>>();

        for s in snapshots {
            if let Some(s) = s {
                snapshot.merge(s);
            }
        }

        if snapshot.is_empty() {
            None
        } else {
            Some(snapshot)
        }
    }
}
