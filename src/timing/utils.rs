use super::Easing;
use std::time::Duration as StdDuration;

pub(crate) fn clamp_progress(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub(crate) fn sample_easing(easing: Easing, progress: f64) -> f64 {
    f64::from(easing.value(clamp_progress(progress) as f32)).clamp(0.0, 1.0)
}

pub(crate) fn sanitize_non_negative(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        0.0
    }
}

pub(crate) fn sanitize_playback_rate(value: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

pub(crate) fn std_duration_from_secs(seconds: f64) -> StdDuration {
    let seconds = sanitize_non_negative(seconds);

    StdDuration::try_from_secs_f64(seconds).unwrap_or(StdDuration::MAX)
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
pub(crate) fn completed_iterations_from(value: f64) -> u32 {
    if !value.is_finite() || value <= 0.0 {
        return 0;
    }

    if value >= f64::from(u32::MAX) {
        return u32::MAX;
    }

    value.floor() as u32
}
