use float_cmp::assert_approx_eq;

use crate::{
    iced_ext::{
        EffectSnapshot, effect_snapshot, should_subscribe, subscription, tick_effect_snapshot,
        update_tick,
    },
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

#[test]
fn effect_snapshot_extracts_widget_friendly_values() {
    let background = iced::Color::from_rgb(0.2, 0.4, 0.6);
    let border = iced::Color::from_rgb(0.8, 0.7, 0.6);
    let text = iced::Color::from_rgb(0.1, 0.2, 0.3);
    let shadow = iced::Shadow {
        color: iced::Color::BLACK,
        offset: iced::Vector::new(1.0, 2.0),
        blur_radius: 3.0,
    };
    let properties = vec![
        (UiProperty::Opacity, PropertyValue::Scalar(0.75)),
        (UiProperty::TranslateX, PropertyValue::Scalar(12.0)),
        (UiProperty::TranslateY, PropertyValue::Scalar(-4.0)),
        (UiProperty::Scale, PropertyValue::Scalar(1.2)),
        (UiProperty::Radius, PropertyValue::Scalar(8.0)),
        (UiProperty::Background, PropertyValue::Color(background)),
        (UiProperty::BorderColor, PropertyValue::Color(border)),
        (UiProperty::TextColor, PropertyValue::Color(text)),
        (UiProperty::Shadow, PropertyValue::Shadow(shadow)),
    ];

    let effects = effect_snapshot(&properties);

    assert_eq!(effects.opacity, Some(0.75));
    assert_eq!(effects.translation, Some(iced::Vector::new(12.0, -4.0)));
    assert_eq!(effects.scale, Some(1.2));
    assert_eq!(effects.radius, Some(8.0));
    assert_eq!(effects.background, Some(background));
    assert_eq!(effects.border_color, Some(border));
    assert_eq!(effects.text_color, Some(text));
    assert_eq!(effects.shadow, Some(shadow));
    assert!(!effects.is_empty());
}

#[test]
fn effect_snapshot_defaults_missing_translation_axis_to_zero() {
    let effects = effect_snapshot(&vec![(UiProperty::TranslateX, PropertyValue::Scalar(6.0))]);

    assert_eq!(effects.translation, Some(iced::Vector::new(6.0, 0.0)));
}

#[test]
fn tick_effect_snapshot_extracts_runtime_tick_properties() {
    let mut runtime = AnimationRuntime::testing();

    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .scale(0.0, 1.0)
            .scale(1.0, 2.0),
    );

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let tick = update_tick(&mut runtime, Instant::now());
    let effects = tick_effect_snapshot(&tick);

    assert_approx_eq!(f32, effects.opacity.unwrap(), 0.5, epsilon = 1e-5);
    assert_approx_eq!(f32, effects.scale.unwrap(), 1.5, epsilon = 1e-5);
}

#[test]
fn empty_effect_snapshot_reports_empty() {
    assert!(EffectSnapshot::from_properties(&Vec::new()).is_empty());
}
