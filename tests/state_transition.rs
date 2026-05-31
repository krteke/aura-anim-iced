//! Public API coverage for state-driven animation.

use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, Duration, OPACITY, PropertySnapshot, PropertyValue,
    StateAnimator, StateTransition, Timeline, Track,
};
use float_cmp::assert_approx_eq;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelState {
    Closed,
    Open,
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
    let timeline = Timeline::track(
        Track::from(OPACITY, 0.0)
            .to(1.0)
            .duration(Duration::from_millis(100.0)),
    );
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
    let timeline = Timeline::track(
        Track::from(OPACITY, 0.0)
            .to(1.0)
            .duration(Duration::from_millis(100.0)),
    );
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
