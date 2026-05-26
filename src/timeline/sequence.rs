use super::{
    TimelineStep,
    duration::{contains_offset, sum_durations},
};
use crate::{property::PropertySnapshot, timing::Duration};

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
                return step.sample_at(offset_dur.checked_sub(cursor_dur)?);
            }

            cursor_dur = end_dur;
        }

        None
    }
}
