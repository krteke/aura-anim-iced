//! Public API coverage for property-driven value change animation.

use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, BehaviorRule, Duration, OPACITY, PropertySnapshot,
    PropertyTransition, PropertyValue, Timing,
};
use float_cmp::assert_approx_eq;

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
fn property_transition_registers_animation_after_value_change() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.25).is_none());
    assert_eq!(transition.current_value(), Some(0.25));
    assert!(runtime.is_idle());

    assert!(transition.transition_to(&mut runtime, 0.25).is_none());
    assert!(runtime.is_idle());

    let registration = transition
        .transition_to(&mut runtime, 1.0)
        .expect("changed value registers animation");

    assert_eq!(transition.current_value(), Some(1.0));
    assert_eq!(registration.properties().map(opacity), Some(0.25));
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let mid_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(mid_tick.properties_for(target).expect("target output")),
        0.625,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[registration.handle()]);
    assert_approx_eq!(
        f32,
        opacity(final_tick.properties_for(target).expect("target output")),
        1.0,
        epsilon = 1e-5
    );
    assert!(runtime.is_idle());
}

#[test]
fn behavior_rule_can_be_reused_for_multiple_targets() {
    let mut runtime = AnimationRuntime::testing();
    let first = AnimationTargetId::new();
    let second = AnimationTargetId::new();
    let rule = BehaviorRule::new(OPACITY).with_timing(Timing::new(80.0));
    let mut first_transition = rule.bind(first);
    let mut second_transition = rule.bind(second);

    assert_eq!(rule.property(), OPACITY);
    assert_eq!(rule.timing(), Timing::new(80.0));
    assert!(first_transition.transition_to(&mut runtime, 0.0).is_none());
    assert!(second_transition.transition_to(&mut runtime, 0.5).is_none());

    let first_registration = first_transition
        .transition_to(&mut runtime, 1.0)
        .expect("first target changed");
    let second_registration = second_transition
        .transition_to(&mut runtime, 1.0)
        .expect("second target changed");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(first).expect("first target output")),
        0.5,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(second).expect("second target output")),
        0.75,
        epsilon = 1e-5
    );
    assert!(tick.completed().is_empty());

    runtime.clock_mut().set_now(Duration::from_millis(80.0));
    let final_tick = runtime.tick();

    assert_eq!(
        final_tick.completed(),
        &[first_registration.handle(), second_registration.handle()]
    );
}

#[test]
fn property_transition_continues_from_running_visual_value() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let first_registration = transition
        .transition_to(&mut runtime, 1.0)
        .expect("initial target change");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let first_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(first_tick.properties_for(target).expect("target output")),
        0.4,
        epsilon = 1e-5
    );

    let replacement = transition
        .transition_to(&mut runtime, 0.2)
        .expect("replacement target change");

    assert_ne!(replacement.handle(), first_registration.handle());
    assert_eq!(runtime.active_count(), 1);
    assert_approx_eq!(
        f32,
        opacity(replacement.properties().expect("replacement output")),
        0.4,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(90.0));
    let continuation_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(
            continuation_tick
                .properties_for(target)
                .expect("target output")
        ),
        0.3,
        epsilon = 1e-5
    );
    assert!(continuation_tick.completed().is_empty());

    runtime.clock_mut().set_now(Duration::from_millis(140.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[replacement.handle()]);
    assert_approx_eq!(
        f32,
        opacity(final_tick.properties_for(target).expect("target output")),
        0.2,
        epsilon = 1e-5
    );
}

#[test]
fn property_transition_can_start_from_explicit_visual_value() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(120.0));

    let registration = transition
        .transition_from_visual(&mut runtime, 0.35, 0.95)
        .expect("explicit visual value starts transition");

    assert_eq!(transition.current_value(), Some(0.95));
    assert_approx_eq!(
        f32,
        opacity(registration.properties().expect("registration output")),
        0.35,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(60.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(target).expect("target output")),
        0.65,
        epsilon = 1e-5
    );
}

#[test]
fn property_transition_handles_runtime_completion() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let registration = transition
        .transition_to(&mut runtime, 1.0)
        .expect("target change");

    assert_eq!(transition.active_handle(), Some(registration.handle()));
    assert!(transition.is_active(&runtime));
    assert!(!transition.handle_completion(&runtime));

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[registration.handle()]);
    assert!(transition.handle_completion(&runtime));
    assert_eq!(transition.active_handle(), None);
    assert!(!transition.is_active(&runtime));
    assert!(!transition.handle_completion(&runtime));
}

#[test]
fn property_transition_retargets_running_animation_to_new_destination() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.retarget_to(&mut runtime, 1.0).is_none());
    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let initial = transition
        .transition_to(&mut runtime, 1.0)
        .expect("initial target change");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(target).expect("target output")),
        0.4,
        epsilon = 1e-5
    );
    assert!(transition.retarget_to(&mut runtime, 1.0).is_none());

    let retargeted = transition
        .retarget_to(&mut runtime, 0.75)
        .expect("running transition retargets");

    assert_ne!(initial.handle(), retargeted.handle());
    assert_eq!(transition.current_value(), Some(0.75));
    assert_eq!(transition.active_handle(), Some(retargeted.handle()));
    assert_eq!(runtime.active_count(), 1);
    assert_approx_eq!(
        f32,
        opacity(retargeted.properties().expect("retarget output")),
        0.4,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(90.0));
    let retarget_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(retarget_tick.properties_for(target).expect("target output")),
        0.575,
        epsilon = 1e-5
    );
}
