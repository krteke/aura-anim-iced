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
