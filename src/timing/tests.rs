use float_cmp::assert_approx_eq;

use super::{
    Delay, Direction, Duration, Easing, FillMode, IterationCount, Timing, TimingPhase,
    TimingSampleState,
};

#[test]
fn duration_and_delay_sanitize_invalid_values() {
    assert_approx_eq!(
        f64,
        Duration::from_millis(-10.0).as_millis(),
        0.0,
        epsilon = 1e-10
    );
    assert_approx_eq!(
        f64,
        Duration::from_millis(f64::NAN).as_millis(),
        0.0,
        epsilon = 1e-10
    );
    assert_approx_eq!(
        f64,
        Duration::from_secs(1.5).as_millis(),
        1_500.0,
        epsilon = 1e-10
    );

    assert_approx_eq!(
        f64,
        Delay::from_millis(-5.0).as_millis(),
        0.0,
        epsilon = 1e-10
    );
    assert_approx_eq!(
        f64,
        Delay::from_secs(0.25).as_millis(),
        250.0,
        epsilon = 1e-10
    );
}

#[test]
fn timing_builder_stores_playback_configuration() {
    let timing = Timing::new(120.0)
        .with_delay(Delay::from_millis(40.0))
        .with_direction(Direction::AlternateReverse)
        .with_fill_mode(FillMode::Both)
        .with_easing(Easing::EaseInOut)
        .with_iterations(IterationCount::count(3))
        .with_playback_rate(2.0);

    assert_approx_eq!(f64, timing.duration.as_millis(), 120.0, epsilon = 1e-10);
    assert_approx_eq!(f64, timing.delay.as_millis(), 40.0, epsilon = 1e-10);
    assert_eq!(timing.direction, Direction::AlternateReverse);
    assert_eq!(timing.fill_mode, FillMode::Both);
    assert_eq!(timing.easing, Easing::EaseInOut);
    assert_eq!(timing.iterations, IterationCount::count(3));
    assert_approx_eq!(f64, timing.playback_rate, 2.0, epsilon = 1e-10);
}

#[test]
fn easing_uses_iced_curves() {
    assert_approx_eq!(
        f64,
        super::sample_easing(Easing::Linear, 0.25),
        0.25,
        epsilon = 1e-10
    );
    assert!(super::sample_easing(Easing::EaseIn, 0.5) < 0.5);
    assert!(super::sample_easing(Easing::EaseOut, 0.5) > 0.5);

    let first_half = super::sample_easing(Easing::EaseInOut, 0.25);
    let second_half = super::sample_easing(Easing::EaseInOut, 0.75);

    assert!(first_half < 0.25);
    assert!(second_half > 0.75);
}

#[test]
fn easing_clamps_invalid_progress() {
    assert_approx_eq!(
        f64,
        super::sample_easing(Easing::Linear, -1.0),
        0.0,
        epsilon = 1e-10
    );
    assert_approx_eq!(
        f64,
        super::sample_easing(Easing::Linear, 2.0),
        1.0,
        epsilon = 1e-10
    );
    assert_approx_eq!(
        f64,
        super::sample_easing(Easing::Linear, f64::NAN),
        0.0,
        epsilon = 1e-10
    );
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
    assert_eq!(before.sample_state, TimingSampleState::BeforeStart);
    assert!(!before.has_sample());
    assert_approx_eq!(f64, before.iteration_progress, 0.0, epsilon = 1e-10);

    let active = timing.normalize_elapsed(175.0);
    assert_eq!(active.phase, TimingPhase::Active);
    assert_eq!(active.sample_state, TimingSampleState::Active);
    assert!(active.has_sample());
    assert_eq!(active.current_iteration_index, Some(1));
    assert_eq!(active.completed_iterations, 1);
    assert_approx_eq!(f64, active.iteration_progress, 0.25, epsilon = 1e-10);
    assert_approx_eq!(f64, active.eased_iteration_progress, 0.25, epsilon = 1e-10);
    assert_approx_eq!(f64, active.active_progress, 1.25, epsilon = 1e-10);

    let after = timing.normalize_elapsed(250.0);
    assert_eq!(after.phase, TimingPhase::AfterEnd);
    assert_eq!(after.sample_state, TimingSampleState::AfterEnd);
    assert!(!after.has_sample());
    assert_eq!(after.current_iteration_index, None);
    assert_eq!(after.completed_iterations, 2);
    assert_approx_eq!(f64, after.iteration_progress, 1.0, epsilon = 1e-10);
    assert_approx_eq!(f64, after.eased_iteration_progress, 1.0, epsilon = 1e-10);
    assert_approx_eq!(f64, after.active_progress, 2.0, epsilon = 1e-10);
}

#[test]
fn delay_normalization_starts_active_sampling_at_delay_boundary() {
    let timing = Timing::new(100.0).with_delay(Delay::from_millis(50.0));

    let before = timing.normalize_elapsed(49.0);
    assert_eq!(before.phase, TimingPhase::BeforeStart);
    assert_eq!(before.sample_state, TimingSampleState::BeforeStart);

    let start = timing.normalize_elapsed(50.0);
    assert_eq!(start.phase, TimingPhase::Active);
    assert_eq!(start.sample_state, TimingSampleState::Active);
    assert_eq!(start.current_iteration_index, Some(0));
    assert_eq!(start.completed_iterations, 0);
    assert_approx_eq!(f64, start.iteration_progress, 0.0, epsilon = 1e-10);
    assert_approx_eq!(f64, start.active_progress, 0.0, epsilon = 1e-10);
}

#[test]
fn zero_duration_normalizes_to_completion_state() {
    let timing = Timing::new(0.0).with_fill_mode(FillMode::Forwards);

    let normalized = timing.normalize_elapsed(0.0);

    assert_eq!(normalized.phase, TimingPhase::AfterEnd);
    assert_eq!(normalized.sample_state, TimingSampleState::ForwardsFill);
    assert_eq!(normalized.current_iteration_index, None);
    assert_eq!(normalized.completed_iterations, 1);
    assert_approx_eq!(f64, normalized.iteration_progress, 1.0, epsilon = 1e-10);
    assert_approx_eq!(
        f64,
        normalized.eased_iteration_progress,
        1.0,
        epsilon = 1e-10
    );
    assert_approx_eq!(f64, normalized.active_progress, 1.0, epsilon = 1e-10);
}

#[test]
fn elapsed_time_normalization_applies_easing() {
    let timing = Timing::new(100.0).with_easing(Easing::EaseIn);

    let normalized = timing.normalize_elapsed(50.0);

    assert_approx_eq!(f64, normalized.iteration_progress, 0.5, epsilon = 1e-10);
    assert_approx_eq!(
        f64,
        normalized.eased_iteration_progress,
        super::sample_easing(Easing::EaseIn, 0.5),
        epsilon = 1e-10
    );
    assert!(normalized.eased_iteration_progress < normalized.iteration_progress);
}

#[test]
fn playback_rate_scales_elapsed_time() {
    let timing = Timing::new(100.0).with_playback_rate(2.0);

    let normalized = timing.normalize_elapsed(25.0);

    assert_eq!(normalized.phase, TimingPhase::Active);
    assert_approx_eq!(f64, normalized.iteration_progress, 0.5, epsilon = 1e-10);
}

#[test]
fn playback_rate_scales_delay_and_active_elapsed_time_together() {
    let timing = Timing::new(100.0)
        .with_delay(Delay::from_millis(50.0))
        .with_playback_rate(2.0);

    let before = timing.normalize_elapsed(20.0);
    assert_eq!(before.phase, TimingPhase::BeforeStart);

    let active = timing.normalize_elapsed(50.0);
    assert_eq!(active.phase, TimingPhase::Active);
    assert_approx_eq!(f64, active.iteration_progress, 0.5, epsilon = 1e-10);
    assert_approx_eq!(f64, active.active_progress, 0.5, epsilon = 1e-10);
}

#[test]
fn invalid_playback_rate_falls_back_to_normal_speed() {
    let timing = Timing::new(100.0).with_playback_rate(0.0);

    assert_approx_eq!(f64, timing.playback_rate, 1.0, epsilon = 1e-10);
    assert_approx_eq!(
        f64,
        timing.normalize_elapsed(25.0).iteration_progress,
        0.25,
        epsilon = 1e-10
    );
}

#[test]
fn fill_mode_none_emits_no_samples_outside_active_interval() {
    let timing = Timing::new(100.0)
        .with_delay(Delay::from_millis(50.0))
        .with_fill_mode(FillMode::None);

    let before = timing.normalize_elapsed(25.0);
    assert_eq!(before.phase, TimingPhase::BeforeStart);
    assert_eq!(before.sample_state, TimingSampleState::BeforeStart);
    assert!(!before.has_sample());
    assert!(!before.is_filled());
    assert_approx_eq!(f64, before.iteration_progress, 0.0, epsilon = 1e-10);

    let after = timing.normalize_elapsed(150.0);
    assert_eq!(after.phase, TimingPhase::AfterEnd);
    assert_eq!(after.sample_state, TimingSampleState::AfterEnd);
    assert!(!after.has_sample());
    assert!(!after.is_filled());
    assert_approx_eq!(f64, after.iteration_progress, 1.0, epsilon = 1e-10);
}

#[test]
fn backwards_fill_emits_the_first_sample_before_start_only() {
    let timing = Timing::new(100.0)
        .with_delay(Delay::from_millis(50.0))
        .with_fill_mode(FillMode::Backwards);

    let before = timing.normalize_elapsed(25.0);
    assert_eq!(before.phase, TimingPhase::BeforeStart);
    assert_eq!(before.sample_state, TimingSampleState::BackwardsFill);
    assert!(before.has_sample());
    assert!(before.is_filled());
    assert_approx_eq!(f64, before.iteration_progress, 0.0, epsilon = 1e-10);
    assert_approx_eq!(f64, before.eased_iteration_progress, 0.0, epsilon = 1e-10);

    let after = timing.normalize_elapsed(150.0);
    assert_eq!(after.sample_state, TimingSampleState::AfterEnd);
    assert!(!after.has_sample());
}

#[test]
fn forwards_fill_emits_the_final_sample_after_end_only() {
    let timing = Timing::new(100.0)
        .with_delay(Delay::from_millis(50.0))
        .with_fill_mode(FillMode::Forwards);

    let before = timing.normalize_elapsed(25.0);
    assert_eq!(before.sample_state, TimingSampleState::BeforeStart);
    assert!(!before.has_sample());

    let after = timing.normalize_elapsed(150.0);
    assert_eq!(after.phase, TimingPhase::AfterEnd);
    assert_eq!(after.sample_state, TimingSampleState::ForwardsFill);
    assert!(after.has_sample());
    assert!(after.is_filled());
    assert_approx_eq!(f64, after.iteration_progress, 1.0, epsilon = 1e-10);
    assert_approx_eq!(f64, after.eased_iteration_progress, 1.0, epsilon = 1e-10);
}

#[test]
fn both_fill_emits_samples_before_start_and_after_end() {
    let timing = Timing::new(100.0)
        .with_delay(Delay::from_millis(50.0))
        .with_fill_mode(FillMode::Both);

    let before = timing.normalize_elapsed(25.0);
    assert_eq!(before.sample_state, TimingSampleState::BackwardsFill);
    assert!(before.has_sample());
    assert!(before.is_filled());

    let active = timing.normalize_elapsed(75.0);
    assert_eq!(active.sample_state, TimingSampleState::Active);
    assert!(active.has_sample());
    assert!(!active.is_filled());

    let after = timing.normalize_elapsed(150.0);
    assert_eq!(after.sample_state, TimingSampleState::ForwardsFill);
    assert!(after.has_sample());
    assert!(after.is_filled());
}

#[test]
fn normal_direction_samples_each_iteration_forward() {
    let timing = Timing::new(100.0)
        .with_direction(Direction::Normal)
        .with_iterations(2);

    let first = timing.normalize_elapsed(25.0);
    assert_eq!(first.current_iteration_index, Some(0));
    assert_eq!(first.completed_iterations, 0);
    assert_approx_eq!(f64, first.iteration_progress, 0.25, epsilon = 1e-10);

    let second = timing.normalize_elapsed(125.0);
    assert_eq!(second.current_iteration_index, Some(1));
    assert_eq!(second.completed_iterations, 1);
    assert_approx_eq!(f64, second.iteration_progress, 0.25, epsilon = 1e-10);
}

#[test]
fn repeated_iterations_preserve_active_progress_and_complete_at_boundary() {
    let timing = Timing::new(100.0)
        .with_iterations(IterationCount::count(3))
        .with_fill_mode(FillMode::Forwards);

    let middle = timing.normalize_elapsed(250.0);
    assert_eq!(middle.phase, TimingPhase::Active);
    assert_eq!(middle.current_iteration_index, Some(2));
    assert_eq!(middle.completed_iterations, 2);
    assert_approx_eq!(f64, middle.iteration_progress, 0.5, epsilon = 1e-10);
    assert_approx_eq!(f64, middle.active_progress, 2.5, epsilon = 1e-10);

    let complete = timing.normalize_elapsed(300.0);
    assert_eq!(complete.phase, TimingPhase::AfterEnd);
    assert_eq!(complete.sample_state, TimingSampleState::ForwardsFill);
    assert_eq!(complete.current_iteration_index, None);
    assert_eq!(complete.completed_iterations, 3);
    assert_approx_eq!(f64, complete.iteration_progress, 1.0, epsilon = 1e-10);
    assert_approx_eq!(f64, complete.active_progress, 3.0, epsilon = 1e-10);
}

#[test]
fn reverse_direction_samples_each_iteration_backward() {
    let timing = Timing::new(100.0)
        .with_direction(Direction::Reverse)
        .with_iterations(2);

    let first = timing.normalize_elapsed(25.0);
    assert_eq!(first.current_iteration_index, Some(0));
    assert_eq!(first.completed_iterations, 0);
    assert_approx_eq!(f64, first.iteration_progress, 0.75, epsilon = 1e-10);

    let second = timing.normalize_elapsed(125.0);
    assert_eq!(second.current_iteration_index, Some(1));
    assert_eq!(second.completed_iterations, 1);
    assert_approx_eq!(f64, second.iteration_progress, 0.75, epsilon = 1e-10);
}

#[test]
fn reverse_direction_repeats_backward_and_completes_at_start_value() {
    let timing = Timing::new(100.0)
        .with_direction(Direction::Reverse)
        .with_fill_mode(FillMode::Forwards)
        .with_iterations(IterationCount::count(2));

    let first_start = timing.normalize_elapsed(0.0);
    assert_eq!(first_start.current_iteration_index, Some(0));
    assert_eq!(first_start.completed_iterations, 0);
    assert_approx_eq!(f64, first_start.iteration_progress, 1.0, epsilon = 1e-10);

    let second_start = timing.normalize_elapsed(100.0);
    assert_eq!(second_start.current_iteration_index, Some(1));
    assert_eq!(second_start.completed_iterations, 1);
    assert_approx_eq!(f64, second_start.iteration_progress, 1.0, epsilon = 1e-10);

    let complete = timing.normalize_elapsed(200.0);
    assert_eq!(complete.phase, TimingPhase::AfterEnd);
    assert_eq!(complete.sample_state, TimingSampleState::ForwardsFill);
    assert_approx_eq!(f64, complete.iteration_progress, 0.0, epsilon = 1e-10);
}

#[test]
fn alternate_direction_switches_odd_iterations_to_reverse() {
    let timing = Timing::new(100.0)
        .with_direction(Direction::Alternate)
        .with_iterations(3);

    let first = timing.normalize_elapsed(25.0);
    assert_eq!(first.current_iteration_index, Some(0));
    assert_eq!(first.completed_iterations, 0);
    assert_approx_eq!(f64, first.iteration_progress, 0.25, epsilon = 1e-10);

    let second = timing.normalize_elapsed(125.0);
    assert_eq!(second.current_iteration_index, Some(1));
    assert_eq!(second.completed_iterations, 1);
    assert_approx_eq!(f64, second.iteration_progress, 0.75, epsilon = 1e-10);

    let third = timing.normalize_elapsed(225.0);
    assert_eq!(third.current_iteration_index, Some(2));
    assert_eq!(third.completed_iterations, 2);
    assert_approx_eq!(f64, third.iteration_progress, 0.25, epsilon = 1e-10);
}

#[test]
fn alternate_reverse_direction_switches_odd_iterations_to_forward() {
    let timing = Timing::new(100.0)
        .with_direction(Direction::AlternateReverse)
        .with_iterations(3);

    let first = timing.normalize_elapsed(25.0);
    assert_eq!(first.current_iteration_index, Some(0));
    assert_eq!(first.completed_iterations, 0);
    assert_approx_eq!(f64, first.iteration_progress, 0.75, epsilon = 1e-10);

    let second = timing.normalize_elapsed(125.0);
    assert_eq!(second.current_iteration_index, Some(1));
    assert_eq!(second.completed_iterations, 1);
    assert_approx_eq!(f64, second.iteration_progress, 0.25, epsilon = 1e-10);

    let third = timing.normalize_elapsed(225.0);
    assert_eq!(third.current_iteration_index, Some(2));
    assert_eq!(third.completed_iterations, 2);
    assert_approx_eq!(f64, third.iteration_progress, 0.75, epsilon = 1e-10);
}

#[test]
fn direction_sampling_applies_before_easing() {
    let timing = Timing::new(100.0)
        .with_direction(Direction::Reverse)
        .with_easing(Easing::EaseIn);

    let normalized = timing.normalize_elapsed(25.0);

    assert_approx_eq!(f64, normalized.iteration_progress, 0.75, epsilon = 1e-10);
    assert_approx_eq!(
        f64,
        normalized.eased_iteration_progress,
        super::sample_easing(Easing::EaseIn, 0.75),
        epsilon = 1e-10
    );
}

#[test]
fn direction_controls_backwards_and_forwards_fill_endpoints() {
    let reverse = Timing::new(100.0)
        .with_delay(Delay::from_millis(50.0))
        .with_direction(Direction::Reverse)
        .with_fill_mode(FillMode::Both);

    let reverse_before = reverse.normalize_elapsed(25.0);
    assert_eq!(
        reverse_before.sample_state,
        TimingSampleState::BackwardsFill
    );
    assert_approx_eq!(f64, reverse_before.iteration_progress, 1.0, epsilon = 1e-10);

    let reverse_after = reverse.normalize_elapsed(150.0);
    assert_eq!(reverse_after.sample_state, TimingSampleState::ForwardsFill);
    assert_approx_eq!(f64, reverse_after.iteration_progress, 0.0, epsilon = 1e-10);

    let alternate = Timing::new(100.0)
        .with_direction(Direction::Alternate)
        .with_fill_mode(FillMode::Forwards)
        .with_iterations(2);

    let alternate_after = alternate.normalize_elapsed(200.0);
    assert_eq!(
        alternate_after.sample_state,
        TimingSampleState::ForwardsFill
    );
    assert_approx_eq!(
        f64,
        alternate_after.iteration_progress,
        0.0,
        epsilon = 1e-10
    );
}

#[test]
fn zero_duration_uses_configured_iteration_count() {
    let timing = Timing::new(0.0)
        .with_iterations(3)
        .with_fill_mode(FillMode::Forwards);

    let normalized = timing.normalize_elapsed(0.0);

    assert_eq!(normalized.phase, TimingPhase::AfterEnd);
    assert_eq!(normalized.sample_state, TimingSampleState::ForwardsFill);
    assert_eq!(normalized.current_iteration_index, None);
    assert_eq!(normalized.completed_iterations, 3);
    assert_approx_eq!(f64, normalized.active_progress, 3.0, epsilon = 1e-10);
}

#[test]
fn direction_sampling_clamps_nan_progress() {
    assert_approx_eq!(
        f64,
        Direction::Normal.sample_progress(0, f64::NAN),
        0.0,
        epsilon = 1e-10
    );
    assert_approx_eq!(
        f64,
        Direction::Reverse.sample_progress(0, f64::NAN),
        1.0,
        epsilon = 1e-10
    );
}

#[test]
fn huge_duration_inputs_saturate() {
    assert!(Duration::from_secs(f64::MAX).as_millis().is_finite());
    assert!(Delay::from_secs(f64::MAX).as_millis().is_finite());
}

#[test]
fn active_duration_overflow_returns_none() {
    let timing = Timing::new(f64::MAX).with_iterations(2);

    assert_eq!(timing.active_duration(), None);
}

#[test]
fn total_duration_overflow_returns_none() {
    let timing = Timing::new(f64::MAX)
        .with_delay(Delay::from_secs(f64::MAX))
        .with_iterations(1);

    assert_eq!(timing.total_duration(), None);
}

#[test]
fn infinite_iterations_with_huge_elapsed_returns_valid_sample() {
    let timing = Timing::new(100.0)
        .with_iterations(IterationCount::infinite())
        .with_playback_rate(2.0);

    let normalized = timing.normalize_elapsed(f64::MAX);

    assert_eq!(normalized.phase, TimingPhase::Active);
    assert_eq!(normalized.sample_state, TimingSampleState::Active);
    assert!(normalized.has_sample());
    assert!(normalized.iteration_progress.is_finite());
    assert!(normalized.eased_iteration_progress.is_finite());
    assert!(normalized.active_progress.is_finite());
}
