//! Public API coverage for property-driven value change animation.

use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, Duration, OPACITY, PropertySnapshot, PropertyTransition,
    PropertyValue, Timing,
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
