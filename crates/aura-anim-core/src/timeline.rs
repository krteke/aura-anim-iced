//! Sequential, parallel, and fixed-duration animation composition.

mod hold;
mod parallel;
mod sequence;

pub use hold::Hold;
pub use parallel::Parallel;
pub use sequence::Sequence;

/// A sequential animation timeline.
pub type Timeline<T> = Sequence<T>;

fn normalized(progress: f32) -> f32 {
    if progress.is_nan() {
        0.0
    } else {
        progress.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::normalized;
    use float_cmp::assert_approx_eq;

    #[test]
    fn normalized_clamps_and_sanitizes_progress() {
        assert_approx_eq!(f32, normalized(f32::NAN), 0.0);
        assert_approx_eq!(f32, normalized(-1.0), 0.0);
        assert_approx_eq!(f32, normalized(0.4), 0.4);
        assert_approx_eq!(f32, normalized(2.0), 1.0);
    }
}
