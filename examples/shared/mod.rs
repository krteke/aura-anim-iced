#![allow(dead_code)]

use aura_anim_iced::{
    iced_ext::{self, EffectSnapshot},
    keyframes::KeyframesBuilder,
    property::{self, PropertySpec, PropertyValue},
    runtime::{AnimationRuntime, AnimationTargetId, AnimationTick},
    timeline::Track,
    timing::Timing,
};
use iced::{Color, Shadow, Vector};
use std::time::Instant;

pub(crate) fn tick_effects(
    runtime: &mut AnimationRuntime,
    tick_instant: Instant,
    target: AnimationTargetId,
) -> EffectSnapshot {
    let tick = iced_ext::update_tick(runtime, tick_instant);

    iced_ext::tick_effect_snapshot_for(&tick, target)
}

pub(crate) fn merge_effects(current: &EffectSnapshot, update: &EffectSnapshot) -> EffectSnapshot {
    EffectSnapshot {
        opacity: update.opacity.or(current.opacity),
        width: update.width.or(current.width),
        height: update.height.or(current.height),
        padding: update.padding.or(current.padding),
        translation: update.translation.or(current.translation),
        scale: update.scale.or(current.scale),
        radius: update.radius.or(current.radius),
        background: update.background.or(current.background),
        border_color: update.border_color.or(current.border_color),
        text_color: update.text_color.or(current.text_color),
        shadow: update.shadow.or(current.shadow),
        rotate: update.rotate.or(current.rotate),
        transform: update.transform.or(current.transform),
    }
}

pub(crate) fn scalar_track(
    spec: PropertySpec<property::Scalar>,
    from: f32,
    to: f32,
    timing: Timing,
) -> Track {
    Track::new(
        KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (spec, from))
            .at(1.0, (spec, to))
            .finish(),
    )
}

pub(crate) fn color_track(
    spec: PropertySpec<property::Color>,
    from: Color,
    to: Color,
    timing: Timing,
) -> Track {
    Track::new(
        KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (spec, from))
            .at(1.0, (spec, to))
            .finish(),
    )
}

pub(crate) fn shadow_track(from: Shadow, to: Shadow, timing: Timing) -> Track {
    Track::new(
        KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (property::SHADOW, from))
            .at(1.0, (property::SHADOW, to))
            .finish(),
    )
}

pub(crate) fn tick_scalar(
    tick: &AnimationTick,
    target: AnimationTargetId,
    spec: PropertySpec<property::Scalar>,
) -> Option<f32> {
    let entry = tick
        .properties_for(target)
        .and_then(|snapshot| snapshot.find_property(&spec.raw()))?;

    match entry.value() {
        PropertyValue::Scalar(value) => Some(*value),
        _ => None,
    }
}

pub(crate) fn card_shadow(alpha: f32, y: f32, blur: f32) -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, alpha),
        offset: Vector::new(0.0, y),
        blur_radius: blur,
    }
}
