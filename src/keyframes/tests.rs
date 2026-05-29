use float_cmp::assert_approx_eq;

use super::{Keyframe, KeyframeSegment, Keyframes};
use crate::{
    property::{
        BACKGROUND, OPACITY, PropertyEntry, PropertyKey, PropertySnapshot, PropertySpec,
        PropertyValue, SCALE, Size, Vector2, WIDTH,
    },
    timing::{Delay, Easing, FillMode, Timing},
};

const OFFSET: PropertySpec<Vector2> = PropertySpec::new(PropertyKey::new("test", "offset"), 20);
const BOX_SIZE: PropertySpec<Size> = PropertySpec::new(PropertyKey::new("test", "size"), 30);
const SCALE_AS_COLOR: PropertySpec<crate::property::Color> =
    PropertySpec::new(PropertyKey::new("aura", "scale"), 20);

fn scalar(snapshot: &PropertySnapshot, spec: PropertySpec<crate::property::Scalar>) -> f32 {
    let Some(entry) = snapshot.find_property(&spec.raw()) else {
        panic!("expected scalar property {}", spec.raw().key().name());
    };
    let PropertyValue::Scalar(value) = entry.value() else {
        panic!("expected scalar value");
    };

    *value
}

#[test]
fn new_keyframes_start_empty_with_default_timing() {
    let keyframes = Keyframes::new();

    assert!(keyframes.is_empty());
    assert_eq!(keyframes.len(), 0);
    assert!(keyframes.frames().is_empty());
    assert_eq!(*keyframes.timing(), Timing::default());
}

#[test]
fn offsets_are_normalized_sorted_and_duplicate_offsets_merge() {
    let keyframes = Keyframes::new()
        .at(1.25, (OPACITY, 1.0))
        .at(-0.25, (OPACITY, 0.0))
        .at(f32::NAN, (SCALE, 1.5))
        .at(0.0, (OPACITY, 0.25));

    let offsets = keyframes
        .frames()
        .iter()
        .map(Keyframe::offset)
        .collect::<Vec<_>>();

    assert_eq!(offsets, vec![0.0, 1.0]);
    assert_approx_eq!(
        f32,
        scalar(keyframes.frames()[0].snapshot(), OPACITY),
        0.25,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        scalar(keyframes.frames()[0].snapshot(), SCALE),
        1.5,
        epsilon = 1e-5
    );
}

#[test]
fn helper_builders_insert_common_typed_properties() {
    let color = iced::Color::from_rgb(0.2, 0.3, 0.4);
    let keyframes = Keyframes::new()
        .background_color(0.5, color)
        .opacity(0.5, 0.8)
        .scale(1.0, 1.2)
        .with_timing(Timing::new(120.0).with_delay(Delay::from_millis(20.0)));

    assert_eq!(keyframes.len(), 2);
    assert_approx_eq!(
        f32,
        scalar(keyframes.frames()[0].snapshot(), OPACITY),
        0.8,
        epsilon = 1e-5
    );
    assert_eq!(
        keyframes.frames()[0]
            .find_property(&BACKGROUND.raw())
            .map(PropertyEntry::value),
        Some(&PropertyValue::Color(color))
    );
    assert_eq!(keyframes.timing().delay(), Delay::from_millis(20.0));
}

#[test]
fn segment_lookup_covers_empty_single_exact_and_between_cases() {
    assert_eq!(Keyframes::new().segment_at(0.5), KeyframeSegment::Empty);

    let single = Keyframes::new().at(0.5, (OPACITY, 0.5));
    assert!(single.segment_at(0.2).is_resolved());

    let keyframes = Keyframes::new()
        .at(0.25, (OPACITY, 0.25))
        .at(0.75, (OPACITY, 0.75));

    let KeyframeSegment::Exact(first) = keyframes.segment_at(0.0) else {
        panic!("expected edge exact segment");
    };
    assert_approx_eq!(f32, first.offset(), 0.25, epsilon = 1e-5);

    let KeyframeSegment::Between { from, to, progress } = keyframes.segment_at(0.5) else {
        panic!("expected interpolating segment");
    };
    assert_approx_eq!(f32, from.offset(), 0.25, epsilon = 1e-5);
    assert_approx_eq!(f32, to.offset(), 0.75, epsilon = 1e-5);
    assert_approx_eq!(f32, progress, 0.5, epsilon = 1e-5);
}

#[test]
fn sample_at_interpolates_scalar_color_vector_and_size_values() {
    let from_color = iced::Color::from_rgb(0.0, 0.2, 0.4);
    let to_color = iced::Color::from_rgb(1.0, 0.6, 0.0);
    let keyframes = Keyframes::new()
        .at(
            0.0,
            PropertySnapshot::from(vec![
                PropertyEntry::new(OPACITY, 0.0),
                PropertyEntry::new(WIDTH, 100.0),
                PropertyEntry::new(OFFSET, iced::Vector::new(0.0, 10.0)),
                PropertyEntry::new(BOX_SIZE, iced::Size::new(20.0, 40.0)),
                PropertyEntry::new(BACKGROUND, from_color),
            ]),
        )
        .at(
            1.0,
            PropertySnapshot::from(vec![
                PropertyEntry::new(OPACITY, 1.0),
                PropertyEntry::new(WIDTH, 200.0),
                PropertyEntry::new(OFFSET, iced::Vector::new(10.0, 30.0)),
                PropertyEntry::new(BOX_SIZE, iced::Size::new(40.0, 80.0)),
                PropertyEntry::new(BACKGROUND, to_color),
            ]),
        );

    let sampled = keyframes.sample_at(0.5).expect("sample");

    assert_approx_eq!(f32, scalar(&sampled, OPACITY), 0.5, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(&sampled, WIDTH), 150.0, epsilon = 1e-5);
    assert_eq!(
        sampled
            .find_property(&OFFSET.raw())
            .map(PropertyEntry::value),
        Some(&PropertyValue::Vector2(iced::Vector::new(5.0, 20.0)))
    );
    assert_eq!(
        sampled
            .find_property(&BOX_SIZE.raw())
            .map(PropertyEntry::value),
        Some(&PropertyValue::Size(iced::Size::new(30.0, 60.0)))
    );
    let Some(PropertyValue::Color(color)) = sampled
        .find_property(&BACKGROUND.raw())
        .map(PropertyEntry::value)
    else {
        panic!("expected color");
    };
    assert_approx_eq!(f32, color.r, 0.5, epsilon = 1e-5);
    assert_approx_eq!(f32, color.g, 0.4, epsilon = 1e-5);
}

#[test]
fn mismatched_value_shapes_drop_that_property_instead_of_panicking() {
    let keyframes = Keyframes::new()
        .at(
            0.0,
            PropertySnapshot::from(vec![PropertyEntry::new(SCALE, 1.0)]),
        )
        .at(
            1.0,
            PropertySnapshot::from(vec![PropertyEntry::new(SCALE_AS_COLOR, iced::Color::WHITE)]),
        );

    let sampled = keyframes.sample_at(0.5).expect("empty sample");

    assert!(sampled.is_empty());
}

#[test]
fn timing_fill_controls_elapsed_sampling() {
    let keyframes = Keyframes::new()
        .with_timing(
            Timing::new(100.0)
                .with_delay(Delay::from_millis(50.0))
                .with_fill_mode(FillMode::None),
        )
        .opacity(0.0, 0.0)
        .opacity(1.0, 1.0);

    let before = keyframes.timing().normalize_elapsed(25.0);
    assert!(!before.has_sample());

    let after = keyframes
        .with_timing(Timing::new(100.0).with_fill_mode(FillMode::Forwards))
        .timing()
        .normalize_elapsed(120.0);
    assert!(after.has_sample());
    assert_approx_eq!(f64, after.iteration_progress, 1.0, epsilon = 1e-10);
}

#[test]
fn easing_is_applied_between_neighboring_keyframes() {
    let keyframes = Keyframes::new()
        .with_timing(Timing::new(100.0).with_easing(Easing::EaseIn))
        .opacity(0.0, 0.0)
        .opacity(1.0, 1.0);

    let sampled = keyframes.sample_at(0.5).expect("sample");

    assert_approx_eq!(
        f32,
        scalar(&sampled, OPACITY),
        Easing::EaseIn.value(0.5),
        epsilon = 1e-5
    );
}
