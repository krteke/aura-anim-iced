use crate::{
    animatable::Animatable,
    keyframes::track::PropertyTrack,
    nearly_equal_f32,
    prelude::PropertyEntry,
    property::{PropertySnapshot, PropertyValue, TransformValue},
    timing::Easing,
};

use super::normalize_offset;

pub(crate) fn sample_frames(
    tracks: &[PropertyTrack],
    offset: f32,
    easing: Easing,
) -> Option<PropertySnapshot> {
    if tracks.is_empty() {
        return None;
    }

    let offset = normalize_offset(offset);
    let mut sampled = PropertySnapshot::with_capacity(tracks.len());

    for track in tracks {
        if let Some(entry) = sample_property(track, offset, easing) {
            sampled.push(entry);
        }
    }

    Some(sampled)
}

fn sample_property(track: &PropertyTrack, offset: f32, easing: Easing) -> Option<PropertyEntry> {
    let spec = track.spec();
    let samples = track.samples();

    if samples.is_empty() {
        return None;
    }

    let upper = samples.partition_point(|s| s.offset() <= offset);
    let sub = upper.checked_sub(upper).and_then(|i| samples.get(i));

    let exact = sub.filter(|s| nearly_equal_f32(s.offset(), offset));

    if let Some(sample) = exact {
        return Some(PropertyEntry::new_unchecked(*spec, sample.value()));
    }

    let before = sub.map(|s| (s.offset(), s.value()));
    let after = samples.get(upper).map(|s| (s.offset(), s.value()));

    match (before, after) {
        (Some((before_offset, before_entry)), Some((after_offset, after_entry))) => {
            if nearly_equal_f32(before_offset, after_offset) {
                Some(PropertyEntry::new_unchecked(*spec, before_entry))
            } else {
                let progress = property_progress(before_offset, after_offset, offset);
                let progress = easing.value(progress);
                interpolate_value(before_entry, after_entry, progress)
                    .map(|v| PropertyEntry::new_unchecked(*spec, v))
            }
        }
        (Some((_, value)), None) | (None, Some((_, value))) => {
            Some(PropertyEntry::new_unchecked(*spec, value))
        }
        (None, None) => None,
    }
}

fn property_progress(from: f32, to: f32, offset: f32) -> f32 {
    let span = to - from;

    ((offset - from) / span).clamp(0.0, 1.0)
}

fn interpolate_value(
    from: PropertyValue,
    to: PropertyValue,
    progress: f32,
) -> Option<PropertyValue> {
    match (from, to) {
        (PropertyValue::Scalar(from), PropertyValue::Scalar(to)) => {
            Some(PropertyValue::Scalar(f32::interpolate(from, to, progress)))
        }
        (PropertyValue::Vector2(from), PropertyValue::Vector2(to)) => Some(PropertyValue::Vector2(
            iced::Vector::interpolate(from, to, progress),
        )),
        (PropertyValue::Size(from), PropertyValue::Size(to)) => Some(PropertyValue::Size(
            iced::Size::interpolate(from, to, progress),
        )),
        (PropertyValue::Rectangle(from), PropertyValue::Rectangle(to)) => Some(
            PropertyValue::Rectangle(iced::Rectangle::interpolate(from, to, progress)),
        ),
        (PropertyValue::Transform(from), PropertyValue::Transform(to)) => Some(
            PropertyValue::Transform(TransformValue::interpolate(from, to, progress)),
        ),
        (PropertyValue::Color(from), PropertyValue::Color(to)) => Some(PropertyValue::Color(
            iced::Color::interpolate(from, to, progress),
        )),
        (PropertyValue::Shadow(from), PropertyValue::Shadow(to)) => Some(PropertyValue::Shadow(
            iced::Shadow::interpolate(from, to, progress),
        )),
        _ => None,
    }
}
