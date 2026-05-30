use std::time::Instant;

use super::{EffectSnapshot, effect_snapshot, should_subscribe, subscription, update_tick};
use crate::{
    keyframes::KeyframesBuilder,
    property::{
        BACKGROUND, BORDER_COLOR, HEIGHT, OPACITY, PADDING, PropertyEntry, PropertySnapshot,
        RADIUS, SCALE, SHADOW, TEXT_COLOR, WIDTH,
    },
    runtime::{AnimationRuntime, AnimationTargetId},
    timing::{Duration, Timing},
};

fn animation_tick(_: Instant) {}

#[test]
fn subscription_gate_tracks_runtime_playing_state() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();

    assert!(!should_subscribe(&runtime));
    let _idle = subscription(&runtime, animation_tick);

    runtime.register_keyframes(
        target,
        KeyframesBuilder::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .finish(),
    );

    assert!(should_subscribe(&runtime));
    let _active = subscription(&runtime, animation_tick);

    runtime.pause_target(target);
    assert!(!should_subscribe(&runtime));
}

#[test]
fn update_tick_routes_iced_tick_into_runtime() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();

    runtime.register_keyframes(
        target,
        KeyframesBuilder::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .finish(),
    );
    runtime.clock_mut().set_now(Duration::from_millis(50.0));

    let tick = update_tick(&mut runtime, Instant::now());

    assert_eq!(tick.timestamp(), Duration::from_millis(50.0));
    assert!(tick.properties_for(target).is_some());
}

#[test]
fn effect_snapshot_extracts_all_supported_builtin_effects() {
    let background = iced::Color::from_rgb(0.1, 0.2, 0.3);
    let border = iced::Color::from_rgb(0.4, 0.5, 0.6);
    let text = iced::Color::from_rgb(0.7, 0.8, 0.9);
    let shadow = iced::Shadow {
        color: iced::Color::BLACK,
        offset: iced::Vector::new(2.0, 4.0),
        blur_radius: 12.0,
    };
    let properties = PropertySnapshot::from(vec![
        PropertyEntry::new(OPACITY, 0.75),
        PropertyEntry::new(WIDTH, 120.0),
        PropertyEntry::new(HEIGHT, 48.0),
        PropertyEntry::new(PADDING, 16.0),
        PropertyEntry::new(SCALE, 1.2),
        PropertyEntry::new(RADIUS, 8.0),
        PropertyEntry::new(BACKGROUND, background),
        PropertyEntry::new(BORDER_COLOR, border),
        PropertyEntry::new(TEXT_COLOR, text),
        PropertyEntry::new(SHADOW, shadow),
    ]);

    let effects = effect_snapshot(&properties);

    assert_eq!(effects.opacity, Some(0.75));
    assert_eq!(effects.width, Some(120.0));
    assert_eq!(effects.height, Some(48.0));
    assert_eq!(effects.padding, Some(16.0));
    assert_eq!(effects.scale, Some(1.2));
    assert_eq!(effects.radius, Some(8.0));
    assert_eq!(effects.background, Some(background));
    assert_eq!(effects.border_color, Some(border));
    assert_eq!(effects.text_color, Some(text));
    assert_eq!(effects.shadow, Some(shadow));
    assert!(effects.translation.is_none());
}

#[test]
fn effect_snapshot_ignores_wrong_value_shapes_and_unknown_properties() {
    let wrong = PropertySnapshot::from(vec![PropertyEntry::new(
        BACKGROUND,
        iced::Color::from_rgb(0.1, 0.2, 0.3),
    )]);
    let mut wrong_shape = PropertySnapshot::from(vec![PropertyEntry::new(OPACITY, 0.5)]);
    wrong_shape.merge(wrong);

    assert_eq!(effect_snapshot(&wrong_shape).opacity, Some(0.5));

    let properties = PropertySnapshot::from(vec![PropertyEntry::new(
        crate::property::PropertySpec::<crate::property::Size>::new(
            crate::property::PropertyKey::new("test", "opacity-like-size"),
            OPACITY.raw().composition_order(),
        ),
        iced::Size::new(1.0, 2.0),
    )]);

    assert!(effect_snapshot(&properties).is_empty());
}

#[test]
fn tick_effect_snapshot_requires_explicit_target() {
    let mut runtime = AnimationRuntime::testing();
    let first = AnimationTargetId::new();
    let second = AnimationTargetId::new();

    runtime.register_keyframes(
        first,
        KeyframesBuilder::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .finish(),
    );
    runtime.register_keyframes(
        second,
        KeyframesBuilder::new()
            .with_timing(Timing::new(100.0))
            .scale(0.0, 1.0)
            .scale(1.0, 2.0)
            .finish(),
    );
    runtime.clock_mut().set_now(Duration::from_millis(50.0));

    let tick = runtime.tick();
    let first_effects = EffectSnapshot::from_tick_for(&tick, first);
    let second_effects = EffectSnapshot::from_tick_for(&tick, second);
    let missing = EffectSnapshot::from_tick_for(&tick, AnimationTargetId::new());

    assert_eq!(first_effects.opacity, Some(0.5));
    assert_eq!(first_effects.scale, None);
    assert_eq!(second_effects.opacity, None);
    assert_eq!(second_effects.scale, Some(1.5));
    assert!(missing.is_empty());
}

#[test]
fn empty_effect_snapshot_reports_empty() {
    assert!(EffectSnapshot::default().is_empty());
    assert!(EffectSnapshot::from_properties(&PropertySnapshot::new()).is_empty());
    assert!(
        !EffectSnapshot {
            opacity: Some(1.0),
            ..EffectSnapshot::default()
        }
        .is_empty()
    );
}
