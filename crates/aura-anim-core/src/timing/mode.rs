use crate::timing::utils::clamp_progress;

/// Playback direction applied to repeated iterations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Play each iteration from start to end.
    #[default]
    Normal,
    /// Play each iteration from end to start.
    Reverse,
    /// Alternate forward and reverse iterations.
    Alternate,
    /// Alternate reverse and forward iterations.
    AlternateReverse,
}

impl Direction {
    pub(crate) fn sample_progress(self, iteration_index: u32, raw_progress: f64) -> f64 {
        let progress = clamp_progress(raw_progress);

        if self.is_reversed_iteration(iteration_index) {
            1.0 - progress
        } else {
            progress
        }
    }

    pub(crate) fn end_progress(self, iteration_count: u32) -> f64 {
        let last_iteration = iteration_count.saturating_sub(1);

        self.sample_progress(last_iteration, 1.0)
    }

    pub(crate) fn is_reversed_iteration(self, iteration_index: u32) -> bool {
        match self {
            Self::Normal => false,
            Self::Reverse => true,
            Self::Alternate => iteration_index % 2 == 1,
            Self::AlternateReverse => iteration_index.is_multiple_of(2),
        }
    }
}
