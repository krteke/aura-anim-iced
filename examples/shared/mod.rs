#![allow(dead_code)]

use aura_anim_iced::{AnimationRuntime, EffectSnapshot, PropertySpec, Track, iced_ext, property};
use iced::{Color, Shadow, Vector};
use std::time::Instant;

pub(crate) fn tick_effects(
    runtime: &mut AnimationRuntime,
    tick_instant: Instant,
    target: aura_anim_iced::AnimationTargetId,
) -> EffectSnapshot {
    let tick = iced_ext::update_tick(runtime, tick_instant);

    aura_anim_iced::tick_effect_snapshot_for(&tick, target)
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
    timing: aura_anim_iced::Timing,
) -> Track {
    Track::new(
        aura_anim_iced::KeyframesBuilder::new()
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
    timing: aura_anim_iced::Timing,
) -> Track {
    Track::new(
        aura_anim_iced::KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (spec, from))
            .at(1.0, (spec, to))
            .finish(),
    )
}

pub(crate) fn shadow_track(from: Shadow, to: Shadow, timing: aura_anim_iced::Timing) -> Track {
    Track::new(
        aura_anim_iced::KeyframesBuilder::new()
            .with_timing(timing)
            .at(0.0, (property::SHADOW, from))
            .at(1.0, (property::SHADOW, to))
            .finish(),
    )
}

pub(crate) fn tick_scalar(
    tick: &aura_anim_iced::AnimationTick,
    target: aura_anim_iced::AnimationTargetId,
    spec: PropertySpec<property::Scalar>,
) -> Option<f32> {
    let entry = tick
        .properties_for(target)
        .and_then(|snapshot| snapshot.find_property(&spec.raw()))?;

    match entry.value() {
        aura_anim_iced::PropertyValue::Scalar(value) => Some(*value),
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
