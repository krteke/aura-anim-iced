use crate::{
    animatable::Animatable,
    nearly_equal_f32,
    property::{
        PropertySnapshot, PropertyValue, TransformValue, UiProperty,
        sort_properties_by_composition, sort_property_entries_by_composition,
    },
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
    let mut sampled = Vec::new();

    for property in unique_properties(frames) {
        if let Some(value) = sample_property(frames, property, offset, easing) {
            sampled.push((property, value));
        }
    }

    sort_property_entries_by_composition(&mut sampled);
    Some(sampled)
}

fn unique_properties(frames: &[Keyframe]) -> Vec<UiProperty> {
    let mut properties = Vec::new();

    for frame in frames {
        for (property, _) in frame.snapshot() {
            if !properties.contains(property) {
                properties.push(*property);
            }
        }
    }

    sort_properties_by_composition(&mut properties);
    properties
}

fn sample_property(
    frames: &[Keyframe],
    property: UiProperty,
    offset: f32,
    easing: Easing,
) -> Option<PropertyValue> {
    let exact = frames
        .iter()
        .find(|frame| nearly_equal_f32(frame.offset(), offset))
        .and_then(|frame| find_property(frame.snapshot(), property));

    if let Some((_, value)) = exact {
        return Some(*value);
    }

    let before = frames
        .iter()
        .rev()
        .filter(|frame| frame.offset() <= offset || nearly_equal_f32(frame.offset(), offset))
        .find_map(|frame| {
            find_property(frame.snapshot(), property).map(|(_, value)| (frame, value))
        });
    let after = frames
        .iter()
        .filter(|frame| frame.offset() >= offset || nearly_equal_f32(frame.offset(), offset))
        .find_map(|frame| {
            find_property(frame.snapshot(), property).map(|(_, value)| (frame, value))
        });

    match (before, after) {
        (Some((before_frame, before_value)), Some((after_frame, after_value))) => {
            if nearly_equal_f32(before_frame.offset(), after_frame.offset()) {
                Some(*before_value)
            } else {
                let progress =
                    property_progress(before_frame.offset(), after_frame.offset(), offset);
                let progress = easing.value(progress);
                interpolate_value(before_value, after_value, progress)
            }
        }
        (Some((_, value)), None) | (None, Some((_, value))) => Some(*value),
        (None, None) => None,
    }
}

fn property_progress(from: f32, to: f32, offset: f32) -> f32 {
    let span = to - from;

    if span > f32::EPSILON {
        ((offset - from) / span).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn find_property(
    snapshot: &PropertySnapshot,
    property: UiProperty,
) -> Option<&(UiProperty, PropertyValue)> {
    snapshot
        .iter()
        .find(|(candidate, _)| *candidate == property)
}

fn interpolate_value(
    from: &PropertyValue,
    to: &PropertyValue,
    progress: f32,
) -> Option<PropertyValue> {
    match (from, to) {
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
            PropertyValue::Transform(interpolate_transform(*from, *to, progress)),
        ),
        (PropertyValue::Color(from), PropertyValue::Color(to)) => Some(PropertyValue::Color(
            iced::Color::interpolate(*from, *to, progress),
        )),
        (PropertyValue::Shadow(from), PropertyValue::Shadow(to)) => Some(PropertyValue::Shadow(
            iced::Shadow::interpolate(*from, *to, progress),
        )),
        _ => {
            // TODO
            None
        }
    }
}

fn interpolate_transform(
    from: TransformValue,
    to: TransformValue,
    progress: f32,
) -> TransformValue {
    TransformValue::new(
        f32::interpolate(from.translate_x, to.translate_x, progress),
        f32::interpolate(from.translate_y, to.translate_y, progress),
        f32::interpolate(from.scale, to.scale, progress),
        f32::interpolate(from.rotate, to.rotate, progress),
    )
}
