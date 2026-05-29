use std::collections::HashSet;

use crate::{
    animatable::Animatable,
    nearly_equal_f32,
    prelude::{PropertyEntry, RawPropertySpec},
    property::{PropertySnapshot, PropertyValue, TransformValue},
    timing::Easing,
};

use super::{Keyframe, normalize_offset};

pub(crate) fn sample_frames(
    frames: &[Keyframe],
    offset: f32,
    easing: Easing,
) -> Option<PropertySnapshot> {
    if frames.is_empty() {
        return None;
    }

    let offset = normalize_offset(offset);
    let mut sampled = PropertySnapshot::new();

    for property in unique_properties(frames) {
        if let Some(entry) = sample_property(frames, property, offset, easing) {
            sampled.push(entry);
        }
    }

    sampled.sort_by_composition_key();
    Some(sampled)
}

fn unique_properties(frames: &[Keyframe]) -> HashSet<RawPropertySpec> {
    let mut properties = HashSet::new();

    for frame in frames {
        for entry in frame.snapshot().entries() {
            properties.insert(*entry.spec());
        }
    }

    properties
}

fn sample_property(
    frames: &[Keyframe],
    property: RawPropertySpec,
    offset: f32,
    easing: Easing,
) -> Option<PropertyEntry> {
    let exact = frames
        .iter()
        .find(|frame| nearly_equal_f32(frame.offset(), offset))
        .and_then(|frame| frame.find_property(&property));

    if let Some(value) = exact {
        return Some(*value);
    }

    let before = frames
        .iter()
        .rev()
        .filter(|frame| frame.offset() <= offset || nearly_equal_f32(frame.offset(), offset))
        .find_map(|frame| frame.find_property(&property).map(|entry| (frame, entry)));
    let after = frames
        .iter()
        .filter(|frame| frame.offset() >= offset || nearly_equal_f32(frame.offset(), offset))
        .find_map(|frame| frame.find_property(&property).map(|entry| (frame, entry)));

    match (before, after) {
        (Some((before_frame, before_entry)), Some((after_frame, after_entry))) => {
            if nearly_equal_f32(before_frame.offset(), after_frame.offset()) {
                Some(*before_entry)
            } else {
                let progress =
                    property_progress(before_frame.offset(), after_frame.offset(), offset);
                let progress = easing.value(progress);
                interpolate_entry(before_entry, after_entry, progress)
            }
        }
        (Some((_, entry)), None) | (None, Some((_, entry))) => Some(*entry),
        (None, None) => None,
    }
}

fn property_progress(from: f32, to: f32, offset: f32) -> f32 {
    let span = to - from;

    ((offset - from) / span).clamp(0.0, 1.0)
}

fn interpolate_entry(
    from: &PropertyEntry,
    to: &PropertyEntry,
    progress: f32,
) -> Option<PropertyEntry> {
    let mut result = *from;

    let value = match (from.value(), to.value()) {
        (PropertyValue::Scalar(from), PropertyValue::Scalar(to)) => Some(PropertyValue::Scalar(
            f32::interpolate(*from, *to, progress),
        )),
        (PropertyValue::Vector2(from), PropertyValue::Vector2(to)) => Some(PropertyValue::Vector2(
            iced::Vector::interpolate(*from, *to, progress),
        )),
        (PropertyValue::Size(from), PropertyValue::Size(to)) => Some(PropertyValue::Size(
            iced::Size::interpolate(*from, *to, progress),
        )),
        (PropertyValue::Rectangle(from), PropertyValue::Rectangle(to)) => Some(
            PropertyValue::Rectangle(iced::Rectangle::interpolate(*from, *to, progress)),
        ),
        (PropertyValue::Transform(from), PropertyValue::Transform(to)) => Some(
            PropertyValue::Transform(TransformValue::interpolate(*from, *to, progress)),
        ),
        (PropertyValue::Color(from), PropertyValue::Color(to)) => Some(PropertyValue::Color(
            iced::Color::interpolate(*from, *to, progress),
        )),
        (PropertyValue::Shadow(from), PropertyValue::Shadow(to)) => Some(PropertyValue::Shadow(
            iced::Shadow::interpolate(*from, *to, progress),
        )),
        _ => None,
    };

    value.map(|value| result.set_value(value))
}
