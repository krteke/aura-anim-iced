use std::time::Duration as StdDuration;

pub(crate) fn clamp_progress(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub(crate) fn sanitize_non_negative(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

pub(crate) fn std_duration_from_secs(seconds: f64) -> StdDuration {
    let seconds = sanitize_non_negative(seconds);

    StdDuration::try_from_secs_f64(seconds).unwrap_or(StdDuration::MAX)
}

#[cfg(test)]
mod tests {
    use super::{clamp_progress, sanitize_non_negative, std_duration_from_secs};
    use float_cmp::assert_approx_eq;

    #[test]
    fn clamp_progress_rejects_invalid_values() {
        assert_approx_eq!(f64, clamp_progress(f64::NAN), 0.0);
        assert_approx_eq!(f64, clamp_progress(f64::INFINITY), 0.0);
        assert_approx_eq!(f64, clamp_progress(f64::NEG_INFINITY), 0.0);
    }

    #[test]
    fn clamp_progress_clamps_to_unit_interval() {
        assert_approx_eq!(f64, clamp_progress(-0.25), 0.0);
        assert_approx_eq!(f64, clamp_progress(0.25), 0.25);
        assert_approx_eq!(f64, clamp_progress(1.25), 1.0);
    }

    #[test]
    fn sanitize_non_negative_keeps_only_positive_finite_values() {
        assert_approx_eq!(f64, sanitize_non_negative(1.5), 1.5);
        assert_approx_eq!(f64, sanitize_non_negative(0.0), 0.0);
        assert_approx_eq!(f64, sanitize_non_negative(-1.0), 0.0);
        assert_approx_eq!(f64, sanitize_non_negative(f64::NAN), 0.0);
    }

    #[test]
    fn std_duration_saturates_and_sanitizes() {
        assert_eq!(std_duration_from_secs(-1.0), std::time::Duration::ZERO);
        assert_eq!(
            std_duration_from_secs(f64::INFINITY),
            std::time::Duration::ZERO
        );
        assert_eq!(std_duration_from_secs(f64::MAX), std::time::Duration::MAX);
    }
}
