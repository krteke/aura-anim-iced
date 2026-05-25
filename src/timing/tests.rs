use super::{Delay, Direction, Duration, FillMode, IterationCount, Timing, TimingPhase};

#[test]
fn duration_and_delay_sanitize_invalid_values() {
    assert_eq!(Duration::from_millis(-10.0).as_millis(), 0.0);
    assert_eq!(Duration::from_millis(f64::NAN).as_millis(), 0.0);
    assert_eq!(Duration::from_secs(1.5).as_millis(), 1_500.0);

    assert_eq!(Delay::from_millis(-5.0).as_millis(), 0.0);
    assert_eq!(Delay::from_secs(0.25).as_millis(), 250.0);
}

#[test]
fn timing_builder_stores_playback_configuration() {
    let timing = Timing::new(120.0)
        .with_delay(Delay::from_millis(40.0))
        .with_direction(Direction::AlternateReverse)
        .with_fill_mode(FillMode::Both)
        .with_iterations(IterationCount::count(3))
        .with_playback_rate(2.0);

    assert_eq!(timing.duration.as_millis(), 120.0);
    assert_eq!(timing.delay.as_millis(), 40.0);
    assert_eq!(timing.direction, Direction::AlternateReverse);
    assert_eq!(timing.fill_mode, FillMode::Both);
    assert_eq!(timing.iterations, IterationCount::count(3));
    assert_eq!(timing.playback_rate, 2.0);
}

#[test]
fn iteration_count_clamps_zero_to_one() {
    assert_eq!(IterationCount::count(0).finite_count(), Some(1));
    assert_eq!(IterationCount::count(2).finite_count(), Some(2));
    assert_eq!(IterationCount::infinite().finite_count(), None);
}

#[test]
fn iteration_count_accepts_u32_builder_input() {
    let timing = Timing::new(100.0).with_iterations(3);

    assert_eq!(timing.iterations.finite_count(), Some(3));
}

#[test]
fn timing_reports_finite_total_duration() {
    let timing = Timing::new(100.0)
        .with_delay(Delay::from_millis(25.0))
        .with_iterations(IterationCount::count(4));

    assert_eq!(
        timing.active_duration().map(Duration::as_millis),
        Some(400.0)
    );
    assert_eq!(
        timing.total_duration().map(Duration::as_millis),
        Some(425.0)
    );
}

#[test]
fn infinite_timing_has_no_finite_total_duration() {
    let timing = Timing::new(100.0).with_iterations(IterationCount::infinite());

    assert_eq!(timing.active_duration(), None);
    assert_eq!(timing.total_duration(), None);
}

#[test]
fn elapsed_time_normalizes_before_active_and_after_end() {
    let timing = Timing::new(100.0)
        .with_delay(Delay::from_millis(50.0))
        .with_iterations(IterationCount::count(2));

    let before = timing.normalize_elapsed(25.0);
    assert_eq!(before.phase, TimingPhase::BeforeStart);
    assert_eq!(before.iteration_progress, 0.0);

    let active = timing.normalize_elapsed(175.0);
    assert_eq!(active.phase, TimingPhase::Active);
    assert_eq!(active.iteration_index, 1);
    assert_eq!(active.iteration_progress, 0.25);
    assert_eq!(active.active_progress, 1.25);

    let after = timing.normalize_elapsed(250.0);
    assert_eq!(after.phase, TimingPhase::AfterEnd);
    assert_eq!(after.iteration_index, 2);
    assert_eq!(after.iteration_progress, 1.0);
    assert_eq!(after.active_progress, 2.0);
}

#[test]
fn playback_rate_scales_elapsed_time() {
    let timing = Timing::new(100.0).with_playback_rate(2.0);

    let normalized = timing.normalize_elapsed(25.0);

    assert_eq!(normalized.phase, TimingPhase::Active);
    assert_eq!(normalized.iteration_progress, 0.5);
}

#[test]
fn invalid_playback_rate_falls_back_to_normal_speed() {
    let timing = Timing::new(100.0).with_playback_rate(0.0);

    assert_eq!(timing.playback_rate, 1.0);
    assert_eq!(timing.normalize_elapsed(25.0).iteration_progress, 0.25);
}
