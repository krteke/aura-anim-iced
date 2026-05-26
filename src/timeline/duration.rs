use crate::{nearly_equal_f64, timing::Duration};

pub(super) fn sum_durations(
    durations: impl IntoIterator<Item = Option<Duration>>,
) -> Option<Duration> {
    let mut total_duration = Duration::ZERO;

    for duration in durations {
        total_duration += duration?;
    }

    Some(total_duration)
}

pub(super) fn max_duration(
    durations: impl IntoIterator<Item = Option<Duration>>,
) -> Option<Duration> {
    let mut max_duration = Duration::ZERO;

    for duration in durations {
        max_duration = max_duration.max(duration?);
    }

    Some(max_duration)
}

pub(super) fn contains_offset(start_ms: f64, end_ms: f64, offset_ms: f64, is_last: bool) -> bool {
    start_ms <= offset_ms
        && (offset_ms < end_ms || (is_last && nearly_equal_f64(offset_ms, end_ms)))
}
