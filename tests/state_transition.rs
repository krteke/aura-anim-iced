//! Public API coverage for state-driven animation.

use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, Duration, OPACITY, PropertySnapshot, PropertyValue,
    StateAnimator, StateTransition, StateTransitionSet, Timeline, Track,
};
use float_cmp::assert_approx_eq;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    assert!(animator.is_active());
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
