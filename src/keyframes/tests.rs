use float_cmp::assert_approx_eq;

use super::{Keyframe, KeyframeSegment, Keyframes};
use crate::{
    property::{PropertyValue, TransformValue, UiProperty},
    timing::{Delay, Easing, Timing},
};

fn snapshot(entries: &[(UiProperty, f32)]) -> Vec<(UiProperty, PropertyValue)> {
    entries
        .iter()
        .map(|(property, value)| (*property, PropertyValue::Scalar(*value)))
        .collect()
}

#[test]
fn new_keyframes_start_empty_with_default_timing() {
    let keyframes = Keyframes::new();

    assert!(keyframes.is_empty());
    assert!(keyframes.frames().is_empty());
    assert_eq!(*keyframes.timing(), Timing::default());
}

#[test]
fn with_timing_attaches_timing_to_track() {
    let timing = Timing::new(250.0).with_delay(Delay::from_millis(50.0));

    let keyframes = Keyframes::new().with_timing(timing);

    assert_eq!(*keyframes.timing(), timing);
}

#[test]
fn at_inserts_keyframes_in_sorted_offset_order() {
    let keyframes = Keyframes::new()
        .at(0.75, snapshot(&[(UiProperty::Opacity, 0.75)]))
        .at(0.25, snapshot(&[(UiProperty::Opacity, 0.25)]))
        .at(0.5, snapshot(&[(UiProperty::Opacity, 0.5)]));

    let offsets: Vec<_> = keyframes.frames().iter().map(Keyframe::offset).collect();

    assert_eq!(offsets, vec![0.25, 0.5, 0.75]);
}

#[test]
fn offsets_are_clamped_and_invalid_offsets_become_zero() {
    let keyframes = Keyframes::new()
        .at(1.25, snapshot(&[(UiProperty::Opacity, 1.0)]))
        .at(-0.5, snapshot(&[(UiProperty::Opacity, 0.0)]))
        .at(f32::NAN, snapshot(&[(UiProperty::Opacity, 0.5)]));

    let offsets: Vec<_> = keyframes.frames().iter().map(Keyframe::offset).collect();

    assert_eq!(offsets, vec![0.0, 0.0, 1.0]);
}

#[test]
fn keyframe_snapshots_are_sorted_by_property_composition() {
    let frame = Keyframe::new(
        0.5,
        snapshot(&[
            (UiProperty::Shadow, 1.0),
            (UiProperty::Opacity, 0.5),
            (UiProperty::Radius, 8.0),
            (UiProperty::TranslateX, 12.0),
        ]),
    );

    let properties: Vec<_> = frame
        .snapshot()
        .iter()
        .map(|(property, _)| *property)
        .collect();

    assert_eq!(
        properties,
        vec![
            UiProperty::Opacity,
            UiProperty::TranslateX,
            UiProperty::Radius,
            UiProperty::Shadow,
        ]
    );
}

#[test]
fn normalize_repairs_manually_mutated_frames() {
    let mut keyframes = Keyframes::from_raw_frames(
        vec![
            Keyframe::new_unchecked(
                2.0,
                snapshot(&[(UiProperty::Shadow, 1.0), (UiProperty::Opacity, 0.0)]),
            ),
            Keyframe::new_unchecked(-1.0, snapshot(&[(UiProperty::Radius, 4.0)])),
        ],
        Timing::default(),
    );

    keyframes.normalize();

    assert_approx_eq!(f32, keyframes.frames()[0].offset(), 0.0, epsilon = 1e-5);
    assert_approx_eq!(f32, keyframes.frames()[1].offset(), 1.0, epsilon = 1e-5);
    assert_eq!(keyframes.frames()[1].snapshot()[0].0, UiProperty::Opacity);
}

#[test]
fn segment_lookup_returns_empty_for_empty_tracks() {
    let keyframes = Keyframes::new();

    assert_eq!(keyframes.segment_at(0.5), KeyframeSegment::Empty);
}

#[test]
fn segment_lookup_returns_single_for_single_frame_tracks() {
    let keyframes = Keyframes::new().at(0.5, snapshot(&[(UiProperty::Opacity, 0.5)]));

    let KeyframeSegment::Single(frame) = keyframes.segment_at(0.25) else {
        panic!("expected single-frame segment");
    };

    assert_approx_eq!(f32, frame.offset(), 0.5, epsilon = 1e-5);
    assert!(keyframes.segment_at(0.75).is_resolved());
}

#[test]
fn segment_lookup_returns_exact_for_track_edges_and_exact_offsets() {
    let keyframes = Keyframes::new()
        .at(0.25, snapshot(&[(UiProperty::Opacity, 0.25)]))
        .at(0.75, snapshot(&[(UiProperty::Opacity, 0.75)]));

    let KeyframeSegment::Exact(before_first) = keyframes.segment_at(-1.0) else {
        panic!("expected first edge exact segment");
    };
    assert_approx_eq!(f32, before_first.offset(), 0.25, epsilon = 1e-5);

    let KeyframeSegment::Exact(exact) = keyframes.segment_at(0.75) else {
        panic!("expected exact segment");
    };
    assert_approx_eq!(f32, exact.offset(), 0.75, epsilon = 1e-6);

    let KeyframeSegment::Exact(after_last) = keyframes.segment_at(2.0) else {
        panic!("expected last edge exact segment");
    };
    assert_approx_eq!(f32, after_last.offset(), 0.75, epsilon = 1e-5);
}

#[test]
fn segment_lookup_returns_between_for_offsets_between_neighbors() {
    let keyframes = Keyframes::new()
        .at(0.25, snapshot(&[(UiProperty::Opacity, 0.25)]))
        .at(0.75, snapshot(&[(UiProperty::Opacity, 0.75)]));

    let KeyframeSegment::Between { from, to, progress } = keyframes.segment_at(0.5) else {
        panic!("expected between segment");
    };

    assert_approx_eq!(f32, from.offset(), 0.25, epsilon = 1e-5);
    assert_approx_eq!(f32, to.offset(), 0.75, epsilon = 1e-5);
    assert_approx_eq!(f32, progress, 0.5, epsilon = 1e-5);
}

#[test]
fn sampling_empty_tracks_returns_none() {
    let keyframes = Keyframes::new();

    assert_eq!(keyframes.sample_at(0.5), None);
}

#[test]
fn sampling_single_and_exact_keyframes_clones_their_snapshots() {
    let keyframes = Keyframes::new().at(0.5, snapshot(&[(UiProperty::Opacity, 0.5)]));

    assert_eq!(
        keyframes.sample_at(0.0),
        Some(snapshot(&[(UiProperty::Opacity, 0.5)]))
    );
    assert_eq!(
        keyframes.sample_at(0.5),
        Some(snapshot(&[(UiProperty::Opacity, 0.5)]))
    );
}

#[test]
fn sampling_between_keyframes_interpolates_scalar_values() {
    let keyframes = Keyframes::new()
        .at(0.25, snapshot(&[(UiProperty::Opacity, 0.0)]))
        .at(0.75, snapshot(&[(UiProperty::Opacity, 1.0)]));

    let sampled = keyframes.sample_at(0.5).expect("sample");

    assert_eq!(sampled.len(), 1);
    assert_eq!(sampled[0].0, UiProperty::Opacity);
    assert_eq!(sampled[0].1, PropertyValue::Scalar(0.5));
}

#[test]
fn sampling_between_keyframes_applies_iced_easing_to_segment_progress() {
    let keyframes = Keyframes::new()
        .with_timing(Timing::new(100.0).with_easing(Easing::EaseIn))
        .at(0.0, snapshot(&[(UiProperty::Opacity, 0.0)]))
        .at(1.0, snapshot(&[(UiProperty::Opacity, 1.0)]));

    let sampled = keyframes.sample_at(0.5).expect("sample");
    let PropertyValue::Scalar(opacity) = sampled[0].1 else {
        panic!("expected scalar opacity");
    };

    assert_approx_eq!(f32, opacity, Easing::EaseIn.value(0.5), epsilon = 1e-5);
}

#[test]
#[allow(clippy::too_many_lines)]
fn sampling_between_keyframes_interpolates_iced_and_transform_values() {
    let from_shadow = iced::Shadow {
        color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        offset: iced::Vector::new(0.0, 4.0),
        blur_radius: 2.0,
    };
    let to_shadow = iced::Shadow {
        color: iced::Color::from_rgba(1.0, 1.0, 1.0, 1.0),
        offset: iced::Vector::new(10.0, 14.0),
        blur_radius: 6.0,
    };
    let keyframes = Keyframes::new()
        .at(
            0.0,
            vec![
                (
                    UiProperty::TranslateX,
                    PropertyValue::Vector2(iced::Vector::new(0.0, 10.0)),
                ),
                (
                    UiProperty::Width,
                    PropertyValue::Size(iced::Size::new(100.0, 200.0)),
                ),
                (
                    UiProperty::Height,
                    PropertyValue::Rectangle(iced::Rectangle {
                        x: 0.0,
                        y: 10.0,
                        width: 100.0,
                        height: 200.0,
                    }),
                ),
                (
                    UiProperty::Scale,
                    PropertyValue::Transform(TransformValue::identity()),
                ),
                (
                    UiProperty::Background,
                    PropertyValue::Color(iced::Color::from_rgb(0.0, 0.5, 1.0)),
                ),
                (UiProperty::Shadow, PropertyValue::Shadow(from_shadow)),
            ],
        )
        .at(
            1.0,
            vec![
                (
                    UiProperty::TranslateX,
                    PropertyValue::Vector2(iced::Vector::new(10.0, 30.0)),
                ),
                (
                    UiProperty::Width,
                    PropertyValue::Size(iced::Size::new(300.0, 600.0)),
                ),
                (
                    UiProperty::Height,
                    PropertyValue::Rectangle(iced::Rectangle {
                        x: 20.0,
                        y: 30.0,
                        width: 300.0,
                        height: 600.0,
                    }),
                ),
                (
                    UiProperty::Scale,
                    PropertyValue::Transform(TransformValue::new(10.0, 20.0, 2.0, 90.0)),
                ),
                (
                    UiProperty::Background,
                    PropertyValue::Color(iced::Color::from_rgb(1.0, 0.5, 0.0)),
                ),
                (UiProperty::Shadow, PropertyValue::Shadow(to_shadow)),
            ],
        );

    let sampled = keyframes.sample_at(0.5).expect("sample");

    assert!(sampled.contains(&(
        UiProperty::TranslateX,
        PropertyValue::Vector2(iced::Vector::new(5.0, 20.0))
    )));
    assert!(sampled.contains(&(
        UiProperty::Width,
        PropertyValue::Size(iced::Size::new(200.0, 400.0))
    )));
    assert!(sampled.contains(&(
        UiProperty::Height,
        PropertyValue::Rectangle(iced::Rectangle {
            x: 10.0,
            y: 20.0,
            width: 200.0,
            height: 400.0,
        })
    )));
    assert!(sampled.contains(&(
        UiProperty::Scale,
        PropertyValue::Transform(TransformValue::new(5.0, 10.0, 1.5, 45.0))
    )));
    assert!(sampled.contains(&(
        UiProperty::Background,
        PropertyValue::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))
    )));

    let sampled_shadow = sampled
        .iter()
        .find_map(|(property, value)| {
            if *property == UiProperty::Shadow {
                Some(value)
            } else {
                None
            }
        })
        .expect("shadow");
    let PropertyValue::Shadow(sampled_shadow) = sampled_shadow else {
        panic!("expected shadow");
    };
    assert_approx_eq!(f32, sampled_shadow.offset.x, 5.0, epsilon = 1e-5);
    assert_approx_eq!(f32, sampled_shadow.offset.y, 9.0, epsilon = 1e-5);
    assert_approx_eq!(f32, sampled_shadow.blur_radius, 4.0, epsilon = 1e-5);
    assert_approx_eq!(f32, sampled_shadow.color.r, 0.5, epsilon = 1e-5);
    assert_approx_eq!(f32, sampled_shadow.color.a, 0.75, epsilon = 1e-5);
}

#[test]
fn sampling_between_keyframes_omits_missing_or_mismatched_properties() {
    let keyframes = Keyframes::new()
        .at(
            0.0,
            vec![
                (UiProperty::Opacity, PropertyValue::Scalar(0.0)),
                (UiProperty::Scale, PropertyValue::Scalar(1.0)),
                (
                    UiProperty::Background,
                    PropertyValue::Color(iced::Color::BLACK),
                ),
            ],
        )
        .at(
            1.0,
            vec![
                (UiProperty::Opacity, PropertyValue::Scalar(1.0)),
                (UiProperty::Scale, PropertyValue::Color(iced::Color::WHITE)),
            ],
        );

    let sampled = keyframes.sample_at(0.5).expect("sample");

    assert_eq!(sampled, snapshot(&[(UiProperty::Opacity, 0.5)]));
}
