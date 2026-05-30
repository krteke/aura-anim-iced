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
    let mut sampled = PropertySnapshot::new();

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

    let exact = samples.binary_search_by(|sample| {
        if nearly_equal_f32(sample.offset(), offset) {
            std::cmp::Ordering::Equal
        } else if sample.offset() < offset {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    if let Ok(index) = exact {
        return Some(PropertyEntry::new_unchecked(*spec, samples[index].value()));
    }

    let before = before_sample(track, offset);
    let after = after_sample(track, offset);

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

fn before_sample(track: &PropertyTrack, offset: f32) -> Option<(f32, PropertyValue)> {
    let samples = track.samples();

    samples
        .partition_point(|sample| sample.offset() < offset)
        .checked_sub(1)
        .and_then(|index| samples.get(index))
        .map(|s| (s.offset(), s.value()))
}

fn after_sample(track: &PropertyTrack, offset: f32) -> Option<(f32, PropertyValue)> {
    let samples = track.samples();

    samples
        .get(samples.partition_point(|sample| sample.offset() <= offset))
        .map(|sample| (sample.offset(), sample.value()))
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
