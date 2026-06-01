//! Integration coverage for target-scoped runtime behavior through the public API.

use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, Duration, Keyframes, KeyframesBuilder, OPACITY,
    PropertySpec, PropertyValue, SCALE, Timing, WIDTH, prelude::PropertyEntry,
    tick_effect_snapshot_for,
};
use float_cmp::assert_approx_eq;

fn keyframes(
    spec: PropertySpec<aura_anim_iced::property::Scalar>,
    from: f32,
    to: f32,
) -> Keyframes {
    KeyframesBuilder::new()
        .with_timing(Timing::new(100.0))
        .at(0.0, (spec, from))
        .at(1.0, (spec, to))
        .finish()
}

#[test]
fn public_runtime_api_keeps_ticks_target_scoped() {
    let mut runtime = AnimationRuntime::testing();
    let panel = AnimationTargetId::new();
    let button = AnimationTargetId::new();

    runtime.register_keyframes(panel, keyframes(OPACITY, 0.0, 1.0));
    runtime.register_keyframes(button, keyframes(OPACITY, 10.0, 20.0));
    runtime.register_keyframes(button, keyframes(SCALE, 1.0, 2.0));
    runtime.clock_mut().set_now(Duration::from_millis(50.0));

    let tick = runtime.tick();

    assert_eq!(tick.properties().targets().count(), 2);
    assert_eq!(tick_effect_snapshot_for(&tick, panel).opacity, Some(0.5));
    assert_eq!(tick_effect_snapshot_for(&tick, panel).scale, None);
    assert_eq!(tick_effect_snapshot_for(&tick, button).opacity, Some(15.0));
    assert_eq!(tick_effect_snapshot_for(&tick, button).scale, Some(1.5));
}

#[test]
fn public_runtime_controls_validate_target_and_handle_pairs() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let other = AnimationTargetId::new();
    let registration = runtime.register_keyframes(target, keyframes(WIDTH, 100.0, 200.0));

    assert!(!runtime.seek(other, registration.handle(), Duration::from_millis(75.0)));
    assert!(runtime.seek(target, registration.handle(), Duration::from_millis(75.0)));

    let tick = runtime.tick();
    let width = tick
        .properties_for(target)
        .and_then(|snapshot| snapshot.find_property(&WIDTH.raw()))
        .map(PropertyEntry::value);

    assert_eq!(width, Some(&PropertyValue::Scalar(175.0)));

    assert!(!runtime.cancel(other, registration.handle()));
    assert!(runtime.cancel(target, registration.handle()));
    assert!(runtime.is_idle());
}

#[test]
fn public_seek_target_and_pause_target_apply_to_all_target_sources() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();

    runtime.register_keyframes(target, keyframes(OPACITY, 0.0, 1.0));
    runtime.register_keyframes(target, keyframes(SCALE, 1.0, 2.0));
    runtime.seek_target(target, Duration::from_millis(25.0));

    let seek_tick = runtime.tick();
    let effects = tick_effect_snapshot_for(&seek_tick, target);

    assert_approx_eq!(f32, effects.opacity.expect("opacity"), 0.25, epsilon = 1e-5);
    assert_approx_eq!(f32, effects.scale.expect("scale"), 1.25, epsilon = 1e-5);

    runtime.pause_target(target);
    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let paused_tick = runtime.tick();
    let paused = tick_effect_snapshot_for(&paused_tick, target);

    assert_eq!(paused.opacity, Some(0.25));
    assert_eq!(paused.scale, Some(1.25));
    assert!(!runtime.should_subscribe());
}
