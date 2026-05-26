//! Compile checks for Iced integration and public prelude usage.
#![cfg(feature = "testing")]
use std::time::Instant;

use aura_anim_iced::{iced_ext, prelude::*};
use float_cmp::assert_approx_eq;

fn animation_tick(_: Instant) {}

fn assert_send_message(_: impl Send + 'static) {}

#[test]
fn public_prelude_drives_iced_integration_helpers() {
    let mut runtime = AnimationRuntime::testing();

    let _idle_subscription = iced_ext::subscription(&runtime, animation_tick);

    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .translation(0.0, 0.0, 4.0)
            .translation(1.0, 10.0, 20.0),
    );

    let _active_subscription = iced_ext::subscription(&runtime, animation_tick);

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let tick = iced_ext::update_tick(&mut runtime, Instant::now());
    let effects = tick_effect_snapshot(&tick);

    assert!(effects.opacity.is_some());
    assert!(effects.translation.is_some());
    assert_send_message(animation_tick);
}

#[test]
fn iced_integration_routes_ticks_updates_runtime_and_reports_idle_output() {
    let mut runtime = AnimationRuntime::testing();

    assert!(!iced_ext::should_subscribe(&runtime));
    let _idle_subscription = iced_ext::subscription(&runtime, animation_tick);

    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .scale(0.0, 1.0)
            .scale(1.0, 2.0),
    );

    assert!(iced_ext::should_subscribe(&runtime));
    let _active_subscription = iced_ext::subscription(&runtime, animation_tick);

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let active_tick = iced_ext::update_tick(&mut runtime, Instant::now());
    let active_effects = tick_effect_snapshot(&active_tick);

    assert_eq!(runtime.active_count(), 1);
    assert!(iced_ext::should_subscribe(&runtime));
    assert_approx_eq!(
        f32,
        active_effects.opacity.expect("active opacity"),
        0.5,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        active_effects.scale.expect("active scale"),
        1.5,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let idle_tick = iced_ext::update_tick(&mut runtime, Instant::now());
    let idle_effects = tick_effect_snapshot(&idle_tick);

    assert_eq!(runtime.active_count(), 0);
    assert!(runtime.is_idle());
    assert!(!iced_ext::should_subscribe(&runtime));
    assert_eq!(idle_tick.completed().len(), 1);
    assert_approx_eq!(
        f32,
        idle_effects.opacity.expect("completion opacity"),
        1.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        idle_effects.scale.expect("completion scale"),
        2.0,
        epsilon = 1e-5
    );
}
