use float_cmp::assert_approx_eq;

use crate::{
    iced_ext::{should_subscribe, subscription, update_tick},
    keyframes::Keyframes,
    property::{PropertySnapshot, PropertyValue, UiProperty},
    runtime::AnimationRuntime,
    timing::{Duration, Timing},
};
use std::time::Instant;

fn scalar(snapshot: &PropertySnapshot, target: UiProperty) -> f32 {
    let Some((_, PropertyValue::Scalar(value))) =
        snapshot.iter().find(|(property, _)| *property == target)
    else {
        panic!("expected scalar property");
    };

    *value
}

#[test]
fn subscription_gate_tracks_runtime_activity() {
    let mut runtime = AnimationRuntime::testing();

    assert!(!should_subscribe(&runtime));

    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0),
    );

    assert!(should_subscribe(&runtime));

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    runtime.tick();

    assert!(!should_subscribe(&runtime));
}

#[test]
fn subscription_helper_compiles_for_idle_and_active_runtime() {
    let mut runtime = AnimationRuntime::testing();

    let _idle = subscription(&runtime, |_| ());

    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0),
    );

    let _active = subscription(&runtime, |_| ());
}

#[test]
fn update_tick_routes_iced_tick_into_runtime_output() {
    let mut runtime = AnimationRuntime::testing();

    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0),
    );

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let tick = update_tick(&mut runtime, Instant::now());

    assert_eq!(tick.timestamp(), Duration::from_millis(50.0));
    assert_eq!(runtime.active_count(), 1);
    assert_approx_eq!(
        f32,
        scalar(tick.properties(), UiProperty::Opacity),
        0.5,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let tick = update_tick(&mut runtime, Instant::now());

    assert_eq!(runtime.active_count(), 0);
    assert_approx_eq!(
        f32,
        scalar(tick.properties(), UiProperty::Opacity),
        1.0,
        epsilon = 1e-5
    );
}
