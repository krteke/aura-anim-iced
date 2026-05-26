use crate::{
    animatable::Animatable,
    property::{
        PropertySnapshot, PropertyValue, TransformValue, UiProperty,
        sort_property_entries_by_composition,
    },
    timing::Easing,
};

use super::{KeyframeSegment, normalize_offset};

pub(crate) fn sample_segment(
    segment: KeyframeSegment<'_>,
    easing: Easing,
) -> Option<PropertySnapshot> {
    match segment {
        KeyframeSegment::Empty => None,
        KeyframeSegment::Single(frame) | KeyframeSegment::Exact(frame) => {
            Some(frame.snapshot().clone())
        }
        KeyframeSegment::Between { from, to, progress } => Some(sample_between(
            from.snapshot(),
            to.snapshot(),
            easing,
            progress,
        )),
    }
}

fn sample_between(
    from: &PropertySnapshot,
    to: &PropertySnapshot,
    easing: Easing,
    progress: f32,
) -> PropertySnapshot {
    let progress = easing.value(normalize_offset(progress));
    let mut sampled = Vec::new();

    for (property, from_value) in from {
        if let Some((_, to_value)) = find_property(to, *property)
            && let Some(value) = interpolate_value(from_value, to_value, progress)
        {
            sampled.push((*property, value));
        }
    }

    sort_property_entries_by_composition(&mut sampled);
    sampled
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
        _ => None,
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
