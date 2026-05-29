use float_cmp::assert_approx_eq;

use super::{
    BACKGROUND, BORDER_COLOR, HEIGHT, OPACITY, PADDING, PropertyEntry, PropertyKey,
    PropertySnapshot, PropertySpec, PropertyValue, RADIUS, SCALE, SHADOW, TEXT_COLOR,
    TransformValue, WIDTH,
};
use crate::property::{Size, Vector2};

const OFFSET: PropertySpec<Vector2> = PropertySpec::new(PropertyKey::new("test", "offset"), 20);
const BOX_SIZE: PropertySpec<Size> = PropertySpec::new(PropertyKey::new("test", "size"), 30);

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
fn built_in_property_keys_and_composition_order_are_stable() {
    let specs = [
        (
            OPACITY.raw().key().name(),
            OPACITY.raw().composition_order(),
        ),
        (SCALE.raw().key().name(), SCALE.raw().composition_order()),
        (WIDTH.raw().key().name(), WIDTH.raw().composition_order()),
        (HEIGHT.raw().key().name(), HEIGHT.raw().composition_order()),
        (
            PADDING.raw().key().name(),
            PADDING.raw().composition_order(),
        ),
        (RADIUS.raw().key().name(), RADIUS.raw().composition_order()),
        (
            BACKGROUND.raw().key().name(),
            BACKGROUND.raw().composition_order(),
        ),
        (
            BORDER_COLOR.raw().key().name(),
            BORDER_COLOR.raw().composition_order(),
        ),
        (
            TEXT_COLOR.raw().key().name(),
            TEXT_COLOR.raw().composition_order(),
        ),
        (SHADOW.raw().key().name(), SHADOW.raw().composition_order()),
    ];

    assert_eq!(
        specs,
        [
            ("opacity", 10),
            ("scale", 20),
            ("width", 30),
            ("height", 31),
            ("padding", 40),
            ("radius", 50),
            ("background", 60),
            ("border-color", 70),
            ("text-color", 80),
            ("shadow", 90),
        ]
    );
    assert_eq!(OPACITY.raw().key().namespace(), "aura");
}

#[test]
fn typed_property_entries_wrap_the_expected_value_shape() {
    let offset = PropertyEntry::new(OFFSET, iced::Vector::new(2.0, 4.0));
    let size = PropertyEntry::new(BOX_SIZE, iced::Size::new(10.0, 20.0));
    let scale = PropertyEntry::new(SCALE, 1.25);

    assert_eq!(offset.spec(), &OFFSET.raw());
    assert_eq!(
        offset.value(),
        &PropertyValue::Vector2(iced::Vector::new(2.0, 4.0))
    );
    assert_eq!(
        size.value(),
        &PropertyValue::Size(iced::Size::new(10.0, 20.0))
    );
    assert_eq!(scale.value(), &PropertyValue::Scalar(1.25));
}

#[test]
fn snapshot_from_typed_pairs_sorts_and_finds_properties() {
    let mut snapshot = PropertySnapshot::from(vec![(SCALE, 2.0), (OPACITY, 0.5)]);

    snapshot.sort_by_composition_key();

    let specs = snapshot
        .entries()
        .iter()
        .map(|entry| entry.spec().key().name())
        .collect::<Vec<_>>();

    assert_eq!(specs, vec!["opacity", "scale"]);
    assert_approx_eq!(f32, scalar(&snapshot, OPACITY), 0.5, epsilon = 1e-5);
    assert_eq!(snapshot.find_property(&WIDTH.raw()), None);
}

#[test]
fn merge_replaces_same_property_and_keeps_distinct_properties() {
    let mut snapshot = PropertySnapshot::from(vec![(OPACITY, 0.25), (SCALE, 1.0)]);

    snapshot.merge(PropertySnapshot::from(vec![
        (OPACITY, 0.75),
        (WIDTH, 120.0),
    ]));

    assert_eq!(snapshot.entries().len(), 3);
    assert_approx_eq!(f32, scalar(&snapshot, OPACITY), 0.75, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(&snapshot, SCALE), 1.0, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(&snapshot, WIDTH), 120.0, epsilon = 1e-5);
}

#[test]
fn mixed_snapshot_preserves_non_scalar_iced_values() {
    let color = iced::Color::from_rgb(0.1, 0.2, 0.3);
    let shadow = iced::Shadow {
        color,
        offset: iced::Vector::new(1.0, 2.0),
        blur_radius: 8.0,
    };
    let snapshot = PropertySnapshot::from(vec![
        PropertyEntry::new(BACKGROUND, color),
        PropertyEntry::new(SHADOW, shadow),
    ]);

    assert_eq!(
        snapshot
            .find_property(&BACKGROUND.raw())
            .map(PropertyEntry::value),
        Some(&PropertyValue::Color(color))
    );
    assert_eq!(
        snapshot
            .find_property(&SHADOW.raw())
            .map(PropertyEntry::value),
        Some(&PropertyValue::Shadow(shadow))
    );
}

#[test]
fn transform_identity_is_stable() {
    assert_eq!(
        TransformValue::identity(),
        TransformValue {
            translate_x: 0.0,
            translate_y: 0.0,
            scale: 1.0,
            rotate: 0.0,
        }
    );
}
