//! Public API coverage for state-driven animation.

use aura_anim_iced::{
    ActiveStateTransition, AnimationRuntime, AnimationTargetId, Duration, OPACITY,
    PropertySnapshot, PropertyValue, StateAnimator, StateTransition, StateTransitionSet, Timeline,
    Track,
};
use float_cmp::assert_approx_eq;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PanelState {
    Closed,
    Open,
    Disabled,
}

fn opacity_timeline(from: f32, to: f32, duration_ms: f64) -> Timeline {
    Timeline::track(
        Track::from(OPACITY, from)
            .to(to)
            .duration(Duration::from_millis(duration_ms)),
    )
}

fn opacity(snapshot: &PropertySnapshot) -> f32 {
    let Some(entry) = snapshot.find_property(&OPACITY.raw()) else {
        panic!("expected opacity property");
    };
    let PropertyValue::Scalar(value) = entry.value() else {
        panic!("expected scalar value");
    };

    *value
}

#[test]
fn state_animator_registers_timeline_for_explicit_state_transition() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let timeline = opacity_timeline(0.0, 1.0, 100.0);
    let transition = StateTransition::new(PanelState::Closed, PanelState::Open, timeline);
    let mut animator = StateAnimator::new(target, PanelState::Closed);

    assert_eq!(transition.from(), PanelState::Closed);
    assert_eq!(transition.to(), PanelState::Open);
    assert_eq!(animator.target(), target);
    assert_eq!(animator.current(), PanelState::Closed);

    let registration = animator
        .transition_with(&mut runtime, &transition)
        .expect("state transition starts timeline");

    assert_eq!(animator.current(), PanelState::Open);
    assert_eq!(animator.active_handle(), Some(registration.handle()));
    assert!(animator.is_active(&runtime));
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(target).expect("target output")),
        0.5,
        epsilon = 1e-5
    );
}

#[test]
fn state_animator_ignores_transition_that_does_not_start_from_current_state() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let timeline = opacity_timeline(0.0, 1.0, 100.0);
    let transition = StateTransition::new(PanelState::Closed, PanelState::Open, timeline);
    let mut animator = StateAnimator::new(target, PanelState::Open);

    assert!(
        animator
            .transition_with(&mut runtime, &transition)
            .is_none()
    );
    assert_eq!(animator.current(), PanelState::Open);
    assert_eq!(animator.active_handle(), None);
    assert!(runtime.is_idle());
}

#[test]
fn state_animator_matches_state_change_to_correct_transition() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transitions = StateTransitionSet::from_transitions([
        StateTransition::new(
            PanelState::Closed,
            PanelState::Open,
            opacity_timeline(0.0, 1.0, 100.0),
        ),
        StateTransition::new(
            PanelState::Open,
            PanelState::Closed,
            opacity_timeline(1.0, 0.0, 200.0),
        ),
    ]);
    let mut animator = StateAnimator::new(target, PanelState::Closed);

    assert_eq!(transitions.transitions().len(), 2);
    assert!(
        transitions
            .find(PanelState::Closed, PanelState::Open)
            .is_some()
    );
    assert!(
        transitions
            .find(PanelState::Closed, PanelState::Disabled)
            .is_none()
    );

    let open = animator
        .transition_to(&mut runtime, PanelState::Open, &transitions)
        .expect("closed to open transition");

    assert_eq!(animator.current(), PanelState::Open);
    assert_eq!(animator.active_handle(), Some(open.handle()));

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let open_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(open_tick.properties_for(target).expect("open output")),
        0.5,
        epsilon = 1e-5
    );

    let close = animator
        .transition_to(&mut runtime, PanelState::Closed, &transitions)
        .expect("open to closed transition");

    assert_ne!(open.handle(), close.handle());
    let replaced = close.replaced().expect("open transition was replaced");
    assert_eq!(replaced.handle(), open.handle());
    assert_eq!(replaced.from(), PanelState::Closed);
    assert_eq!(replaced.to(), PanelState::Open);
    assert_eq!(runtime.active_count(), 1);
    assert_eq!(animator.current(), PanelState::Closed);
    assert_eq!(animator.active_handle(), Some(close.handle()));

    runtime.clock_mut().set_now(Duration::from_millis(150.0));
    let close_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(close_tick.properties_for(target).expect("close output")),
        0.5,
        epsilon = 1e-5
    );

    assert!(
        animator
            .transition_to(&mut runtime, PanelState::Disabled, &transitions)
            .is_none()
    );
    assert_eq!(animator.current(), PanelState::Closed);
}

#[test]
fn state_animator_uses_fallback_when_no_custom_transition_matches() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transitions = StateTransitionSet::from_transitions([StateTransition::new(
        PanelState::Closed,
        PanelState::Open,
        opacity_timeline(0.0, 1.0, 100.0),
    )])
    .with_fallback(opacity_timeline(0.2, 0.8, 120.0));
    let mut animator = StateAnimator::new(target, PanelState::Open);

    assert!(transitions.fallback().is_some());
    assert!(
        transitions
            .find(PanelState::Open, PanelState::Disabled)
            .is_none()
    );

    let registration = animator
        .transition_to(&mut runtime, PanelState::Disabled, &transitions)
        .expect("fallback transition starts");

    assert_eq!(animator.current(), PanelState::Disabled);
    assert_eq!(animator.active_handle(), Some(registration.handle()));
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(60.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(target).expect("fallback output")),
        0.5,
        epsilon = 1e-5
    );
}

#[test]
fn state_animator_tracks_active_transition_progress() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transition = StateTransition::new(
        PanelState::Closed,
        PanelState::Open,
        opacity_timeline(0.0, 1.0, 200.0),
    );
    let mut animator = StateAnimator::new(target, PanelState::Closed);

    let registration = animator
        .transition_with(&mut runtime, &transition)
        .expect("state transition starts");
    let active = animator
        .active_transition()
        .expect("active transition metadata");

    assert_eq!(active.handle(), registration.handle());
    assert_eq!(active.from(), PanelState::Closed);
    assert_eq!(active.to(), PanelState::Open);
    assert_eq!(active.started_at(), Duration::ZERO);
    assert_eq!(active.duration(), Some(Duration::from_millis(200.0)));

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let tick = runtime.tick();
    let progress = animator
        .active_progress_at(tick.timestamp())
        .expect("active transition progress");

    assert_eq!(progress.handle(), registration.handle());
    assert_eq!(progress.from(), PanelState::Closed);
    assert_eq!(progress.to(), PanelState::Open);
    assert_eq!(progress.elapsed(), Duration::from_millis(50.0));
    assert_eq!(progress.duration(), Some(Duration::from_millis(200.0)));
    assert_approx_eq!(f32, progress.progress().expect("finite progress"), 0.25);

    runtime.clock_mut().set_now(Duration::from_millis(250.0));
    let over_tick = runtime.tick();
    let over_progress = animator
        .active_progress_at(over_tick.timestamp())
        .expect("active transition progress");

    assert_eq!(over_progress.progress(), Some(1.0));
}

#[test]
fn state_animator_handles_transition_completion() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transition = StateTransition::new(
        PanelState::Closed,
        PanelState::Open,
        opacity_timeline(0.0, 1.0, 100.0),
    );
    let mut animator = StateAnimator::new(target, PanelState::Closed);

    let registration = animator
        .transition_with(&mut runtime, &transition)
        .expect("state transition starts");

    assert_eq!(animator.active_handle(), Some(registration.handle()));
    assert!(animator.is_active(&runtime));
    assert!(!animator.handle_completion(&runtime));

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[registration.handle()]);
    assert!(animator.handle_completion(&runtime));
    assert_eq!(animator.current(), PanelState::Open);
    assert_eq!(animator.active_handle(), None);
    assert_eq!(animator.active_transition(), None);
    assert_eq!(animator.active_progress_at(final_tick.timestamp()), None);
    assert!(!animator.is_active(&runtime));
    assert!(!animator.handle_completion(&runtime));
}

#[test]
fn state_animator_clears_active_transition_after_external_cancel() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transition = StateTransition::new(
        PanelState::Closed,
        PanelState::Open,
        opacity_timeline(0.0, 1.0, 100.0),
    );
    let mut animator = StateAnimator::new(target, PanelState::Closed);

    let registration = animator
        .transition_with(&mut runtime, &transition)
        .expect("state transition starts");

    assert!(animator.is_active(&runtime));

    assert!(runtime.cancel(target, registration.handle()));
    assert!(!animator.is_active(&runtime));
    assert!(animator.handle_completion(&runtime));
    assert_eq!(animator.active_handle(), None);
    assert_eq!(animator.active_transition(), None);
}

#[test]
fn state_animator_refreshes_stale_active_cache_before_new_transition() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transitions = StateTransitionSet::from_transitions([
        StateTransition::new(
            PanelState::Closed,
            PanelState::Open,
            opacity_timeline(0.0, 1.0, 100.0),
        ),
        StateTransition::new(
            PanelState::Open,
            PanelState::Closed,
            opacity_timeline(1.0, 0.0, 100.0),
        ),
    ]);
    let mut animator = StateAnimator::new(target, PanelState::Closed);

    let open = animator
        .transition_to(&mut runtime, PanelState::Open, &transitions)
        .expect("open transition starts");

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[open.handle()]);
    assert_eq!(animator.active_handle(), Some(open.handle()));
    assert!(!animator.is_active(&runtime));

    let close = animator
        .transition_to(&mut runtime, PanelState::Closed, &transitions)
        .expect("close transition starts after stale active cache is refreshed");

    assert_ne!(open.handle(), close.handle());
    assert_eq!(animator.active_handle(), Some(close.handle()));
    assert!(animator.is_active(&runtime));
    assert_eq!(animator.current(), PanelState::Closed);
}

#[test]
fn state_animator_cleans_replaced_transition_after_replacement_starts() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transitions = StateTransitionSet::from_transitions([
        StateTransition::new(
            PanelState::Closed,
            PanelState::Open,
            opacity_timeline(0.0, 1.0, 100.0),
        ),
        StateTransition::new(
            PanelState::Open,
            PanelState::Closed,
            opacity_timeline(1.0, 0.0, 100.0),
        ),
    ]);
    let mut animator = StateAnimator::new(target, PanelState::Closed);

    let open = animator
        .transition_to(&mut runtime, PanelState::Open, &transitions)
        .expect("open transition starts");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    runtime.tick();

    let close = animator
        .transition_to(&mut runtime, PanelState::Closed, &transitions)
        .expect("close replacement starts");

    assert_eq!(
        close.replaced().map(ActiveStateTransition::handle),
        Some(open.handle())
    );
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let original_end_tick = runtime.tick();

    assert!(original_end_tick.completed().is_empty());
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(140.0));
    let replacement_end_tick = runtime.tick();

    assert_eq!(replacement_end_tick.completed(), &[close.handle()]);
    assert!(runtime.is_idle());
}

#[test]
fn repeated_state_changes_interrupt_active_transition_and_continue_with_latest_state() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transitions = StateTransitionSet::from_transitions([
        StateTransition::new(
            PanelState::Closed,
            PanelState::Open,
            opacity_timeline(0.0, 1.0, 100.0),
        ),
        StateTransition::new(
            PanelState::Open,
            PanelState::Disabled,
            opacity_timeline(1.0, 0.2, 120.0),
        ),
        StateTransition::new(
            PanelState::Disabled,
            PanelState::Closed,
            opacity_timeline(0.2, 0.8, 80.0),
        ),
    ]);
    let mut animator = StateAnimator::new(target, PanelState::Closed);

    let open = animator
        .transition_to(&mut runtime, PanelState::Open, &transitions)
        .expect("open transition starts");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    runtime.tick();

    let disabled = animator
        .transition_to(&mut runtime, PanelState::Disabled, &transitions)
        .expect("second state change interrupts open transition");

    assert_eq!(
        disabled.replaced().map(ActiveStateTransition::handle),
        Some(open.handle())
    );
    assert_eq!(animator.current(), PanelState::Disabled);
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let disabled_tick = runtime.tick();

    assert!(disabled_tick.completed().is_empty());
    assert_approx_eq!(
        f32,
        opacity(
            disabled_tick
                .properties_for(target)
                .expect("disabled output")
        ),
        0.6,
        epsilon = 1e-5
    );

    let closed = animator
        .transition_to(&mut runtime, PanelState::Closed, &transitions)
        .expect("third state change interrupts disabled transition");

    assert_eq!(
        closed.replaced().map(ActiveStateTransition::handle),
        Some(disabled.handle())
    );
    assert_eq!(animator.current(), PanelState::Closed);
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(160.0));
    let stale_tick = runtime.tick();

    assert!(
        !stale_tick.completed().contains(&open.handle()),
        "first interrupted state transition should be canceled"
    );
    assert!(
        !stale_tick.completed().contains(&disabled.handle()),
        "second interrupted state transition should be canceled"
    );
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(180.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[closed.handle()]);
    assert!(runtime.is_idle());
}

#[test]
fn state_transition_set_matches_multiple_state_pairs_to_distinct_timelines() {
    let mut runtime = AnimationRuntime::testing();
    let first_target = AnimationTargetId::new();
    let second_target = AnimationTargetId::new();
    let transitions = StateTransitionSet::from_transitions([
        StateTransition::new(
            PanelState::Closed,
            PanelState::Open,
            opacity_timeline(0.0, 1.0, 100.0),
        ),
        StateTransition::new(
            PanelState::Open,
            PanelState::Disabled,
            opacity_timeline(1.0, 0.2, 240.0),
        ),
        StateTransition::new(
            PanelState::Disabled,
            PanelState::Closed,
            opacity_timeline(0.2, 0.8, 300.0),
        ),
    ]);

    assert_eq!(
        transitions
            .find(PanelState::Open, PanelState::Disabled)
            .expect("open to disabled transition")
            .timeline()
            .total_duration(),
        Some(Duration::from_millis(240.0))
    );
    assert_eq!(
        transitions
            .find(PanelState::Disabled, PanelState::Closed)
            .expect("disabled to closed transition")
            .timeline()
            .total_duration(),
        Some(Duration::from_millis(300.0))
    );

    let mut open_animator = StateAnimator::new(first_target, PanelState::Open);
    let mut disabled_animator = StateAnimator::new(second_target, PanelState::Disabled);

    open_animator
        .transition_to(&mut runtime, PanelState::Disabled, &transitions)
        .expect("open to disabled matches exact pair");
    disabled_animator
        .transition_to(&mut runtime, PanelState::Closed, &transitions)
        .expect("disabled to closed matches exact pair");

    runtime.clock_mut().set_now(Duration::from_millis(120.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(first_target).expect("first output")),
        0.6,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(second_target).expect("second output")),
        0.44,
        epsilon = 1e-5
    );
}

#[test]
fn state_transition_set_keeps_first_duplicate_match() {
    let transitions = StateTransitionSet::from_transitions([
        StateTransition::new(
            PanelState::Closed,
            PanelState::Open,
            opacity_timeline(0.0, 1.0, 100.0),
        ),
        StateTransition::new(
            PanelState::Closed,
            PanelState::Open,
            opacity_timeline(0.0, 0.5, 100.0),
        ),
    ]);

    let transition = transitions
        .find(PanelState::Closed, PanelState::Open)
        .expect("duplicate transition match");
    let sample = transition
        .timeline()
        .sample_at(Duration::from_millis(50.0))
        .expect("sample");

    assert_approx_eq!(f32, opacity(&sample), 0.5, epsilon = 1e-5);
}
