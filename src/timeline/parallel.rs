use super::{TimelineStep, duration::max_duration};
use crate::{property::PropertySnapshot, timing::Duration};

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

    /// Returns the finite maximum step duration, or `None` if any step is infinite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        max_duration(self.steps.iter().map(TimelineStep::total_duration))
    }

    /// Samples this parallel group at local timeline `offset`.
    #[must_use]
    pub fn sample_at(&self, _offset: impl Into<Duration>) -> Option<PropertySnapshot> {
        None
    }
}
