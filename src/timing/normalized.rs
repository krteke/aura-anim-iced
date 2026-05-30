use super::FillMode;

/// The broad phase produced by elapsed-time normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingPhase {
    /// Elapsed time is still before the active interval.
    BeforeStart,
    /// Elapsed time is inside the active interval.
    Active,
    /// Elapsed time is after a finite active interval.
    AfterEnd,
}

/// The sampling state produced by fill-mode-aware timing normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimingSampleState {
    /// Elapsed time is before the active interval and no sample should be emitted.
    BeforeStart,
    /// Elapsed time is inside the active interval.
    Active,
    /// Elapsed time is after the active interval and no sample should be emitted.
    AfterEnd,
    /// Elapsed time is before the active interval and should emit the first sample.
    BackwardsFill,
    /// Elapsed time is after the active interval and should emit the final sample.
    ForwardsFill,
}

impl TimingSampleState {
    /// Returns whether this state should emit a sample.
    #[must_use]
    pub const fn has_sample(self) -> bool {
        matches!(
            self,
            Self::Active | Self::BackwardsFill | Self::ForwardsFill
        )
    }

    /// Returns whether this state is produced by fill behavior.
    #[must_use]
    pub const fn is_filled(self) -> bool {
        matches!(self, Self::BackwardsFill | Self::ForwardsFill)
    }
}

/// Normalized timing coordinates for sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedTiming {
    /// Current timing phase.
    pub phase: TimingPhase,
    /// Fill-mode-aware sampling state.
    pub sample_state: TimingSampleState,
    /// Current zero-based iteration index while active.
    pub current_iteration_index: Option<u32>,
    /// Number of completed iterations.
    pub completed_iterations: u32,
    /// Normalized progress inside the current iteration.
    pub iteration_progress: f64,
    /// Unclamped progress across active iterations.
    pub active_progress: f64,
}

impl NormalizedTiming {
    pub(crate) fn before_start(fill_mode: FillMode, iteration_progress: f64) -> Self {
        let sample_state = if fill_mode.fills_before_start() {
            TimingSampleState::BackwardsFill
        } else {
            TimingSampleState::BeforeStart
        };

        Self {
            phase: TimingPhase::BeforeStart,
            sample_state,
            current_iteration_index: None,
            completed_iterations: 0,
            iteration_progress,
            active_progress: 0.0,
        }
    }

    pub(crate) fn active(
        current_iteration_index: u32,
        iteration_progress: f64,
        active_progress: f64,
    ) -> Self {
        Self {
            phase: TimingPhase::Active,
            sample_state: TimingSampleState::Active,
            current_iteration_index: Some(current_iteration_index),
            completed_iterations: current_iteration_index,
            iteration_progress,
            active_progress,
        }
    }

    pub(crate) fn after_end(
        iteration_count: u32,
        fill_mode: FillMode,
        iteration_progress: f64,
    ) -> Self {
        let sample_state = if fill_mode.fills_after_end() {
            TimingSampleState::ForwardsFill
        } else {
            TimingSampleState::AfterEnd
        };

        Self {
            phase: TimingPhase::AfterEnd,
            sample_state,
            current_iteration_index: None,
            completed_iterations: iteration_count,
            iteration_progress,
            active_progress: f64::from(iteration_count),
        }
    }

    /// Returns whether this normalized timing state should emit a sample.
    #[must_use]
    pub const fn has_sample(self) -> bool {
        self.sample_state.has_sample()
    }

    /// Returns whether this normalized timing state is produced by fill behavior.
    #[must_use]
    pub const fn is_filled(self) -> bool {
        self.sample_state.is_filled()
    }
}
