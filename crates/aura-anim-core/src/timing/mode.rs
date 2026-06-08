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

#[cfg(test)]
mod tests {
    use super::Direction;
    use float_cmp::assert_approx_eq;

    #[test]
    fn sample_progress_applies_direction() {
        assert_approx_eq!(f64, Direction::Normal.sample_progress(0, 0.25), 0.25);
        assert_approx_eq!(f64, Direction::Reverse.sample_progress(0, 0.25), 0.75);
        assert_approx_eq!(f64, Direction::Alternate.sample_progress(1, 0.25), 0.75);
        assert_approx_eq!(
            f64,
            Direction::AlternateReverse.sample_progress(0, 0.25),
            0.75
        );
    }

    #[test]
    fn end_progress_uses_last_iteration_direction() {
        assert_approx_eq!(f64, Direction::Normal.end_progress(3), 1.0);
        assert_approx_eq!(f64, Direction::Reverse.end_progress(3), 0.0);
        assert_approx_eq!(f64, Direction::Alternate.end_progress(2), 0.0);
        assert_approx_eq!(f64, Direction::AlternateReverse.end_progress(2), 1.0);
    }

    #[test]
    fn sample_progress_clamps_raw_progress() {
        assert_approx_eq!(f64, Direction::Normal.sample_progress(0, -1.0), 0.0);
        assert_approx_eq!(f64, Direction::Normal.sample_progress(0, 2.0), 1.0);
        assert_approx_eq!(f64, Direction::Normal.sample_progress(0, f64::NAN), 0.0);
    }
}
