use crate::property::error::PropertyKindError;

use super::{
    PropertyCompositionKey, PropertyValue, PropertyValueKind, TransformValue, UiProperty,
    UiPropertyCategory, sort_properties_by_composition, sort_property_entries_by_composition,
};

#[test]
fn property_ids_are_stable() {
    let ids = UiProperty::ALL.map(UiProperty::id);

    assert_eq!(ids, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);
}

#[test]
fn property_categories_cover_core_visual_groups() {
    assert_eq!(UiProperty::Opacity.category(), UiPropertyCategory::Opacity);
    assert_eq!(
        UiProperty::TranslateX.category(),
        UiPropertyCategory::Transform
    );
    assert_eq!(UiProperty::Scale.category(), UiPropertyCategory::Transform);
    assert_eq!(UiProperty::Width.category(), UiPropertyCategory::Size);
    assert_eq!(UiProperty::Radius.category(), UiPropertyCategory::Radius);
    assert_eq!(UiProperty::Background.category(), UiPropertyCategory::Color);
    assert_eq!(UiProperty::Shadow.category(), UiPropertyCategory::Shadow);
}

#[test]
fn property_composition_order_is_stable() {
    let expected = [
        (UiProperty::Opacity, 10),
        (UiProperty::TranslateX, 20),
        (UiProperty::TranslateY, 20),
        (UiProperty::Scale, 20),
        (UiProperty::Rotate, 20),
        (UiProperty::Width, 30),
        (UiProperty::Height, 30),
        (UiProperty::Padding, 40),
        (UiProperty::Radius, 50),
        (UiProperty::Background, 60),
        (UiProperty::BorderColor, 70),
        (UiProperty::TextColor, 80),
        (UiProperty::Shadow, 90),
    ];

    for (property, order) in expected {
        assert_eq!(property.composition_order(), order);
    }
}

#[test]
fn property_composition_key_uses_order_then_stable_id() {
    assert_eq!(
        UiProperty::TranslateY.composition_key(),
        PropertyCompositionKey::new(20, 3)
    );
    assert!(UiProperty::TranslateX.composition_key() < UiProperty::Scale.composition_key());
    assert!(UiProperty::Opacity.composition_key() < UiProperty::Shadow.composition_key());
}

#[test]
fn properties_sort_by_composition_order() {
    let mut properties = [
        UiProperty::Shadow,
        UiProperty::TextColor,
        UiProperty::Opacity,
        UiProperty::Background,
        UiProperty::Scale,
        UiProperty::Radius,
        UiProperty::BorderColor,
        UiProperty::Width,
    ];

    sort_properties_by_composition(&mut properties);

    assert_eq!(
        properties,
        [
            UiProperty::Opacity,
            UiProperty::Scale,
            UiProperty::Width,
            UiProperty::Radius,
            UiProperty::Background,
            UiProperty::BorderColor,
            UiProperty::TextColor,
            UiProperty::Shadow,
        ]
    );
}

#[test]
fn property_value_entries_sort_by_composition_order() {
    let mut entries = [
        (UiProperty::Shadow, PropertyValue::Scalar(9.0)),
        (UiProperty::Opacity, PropertyValue::Scalar(1.0)),
        (UiProperty::Radius, PropertyValue::Scalar(5.0)),
        (UiProperty::TranslateX, PropertyValue::Scalar(2.0)),
    ];

    sort_property_entries_by_composition(&mut entries);

    let properties = entries.map(|(property, _)| property);
    assert_eq!(
        properties,
        [
            UiProperty::Opacity,
            UiProperty::TranslateX,
            UiProperty::Radius,
            UiProperty::Shadow,
        ]
    );
}

#[test]
fn parallel_property_storage_preserves_property_value_pairs() {
    let mut entries = vec![
        (UiProperty::TranslateY, PropertyValue::Scalar(24.0)),
        (UiProperty::Opacity, PropertyValue::Scalar(0.5)),
        (UiProperty::Scale, PropertyValue::Scalar(1.2)),
        (UiProperty::TranslateX, PropertyValue::Scalar(12.0)),
    ];

    for (property, value) in &entries {
        assert!(property.accepts_value(value));
    }

    sort_property_entries_by_composition(&mut entries);

    assert_eq!(
        entries,
        vec![
            (UiProperty::Opacity, PropertyValue::Scalar(0.5)),
            (UiProperty::TranslateX, PropertyValue::Scalar(12.0)),
            (UiProperty::TranslateY, PropertyValue::Scalar(24.0)),
            (UiProperty::Scale, PropertyValue::Scalar(1.2)),
        ]
    );
}

#[test]
fn property_values_cover_core_value_shapes() {
    assert_eq!(PropertyValue::Scalar(0.5), PropertyValue::Scalar(0.5));
    assert_eq!(
        PropertyValue::Vector2(iced::Vector::new(1.0, 2.0)),
        PropertyValue::Vector2(iced::Vector { x: 1.0, y: 2.0 })
    );
    assert_eq!(
        PropertyValue::Size(iced::Size::new(10.0, 20.0)),
        PropertyValue::Size(iced::Size {
            width: 10.0,
            height: 20.0,
        })
    );
    assert_eq!(
        PropertyValue::Rectangle(iced::Rectangle::new(
            iced::Point::new(1.0, 2.0),
            iced::Size::new(3.0, 4.0),
        )),
        PropertyValue::Rectangle(iced::Rectangle {
            x: 1.0,
            y: 2.0,
            width: 3.0,
            height: 4.0,
        })
    );
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

#[test]
fn property_accepts_matching_scalar_values() {
    let scalar = PropertyValue::Scalar(0.5);

    assert!(UiProperty::Opacity.accepts_value(&scalar));
    assert!(UiProperty::TranslateX.accepts_value(&scalar));
    assert!(UiProperty::Scale.accepts_value(&scalar));
    assert!(UiProperty::Width.accepts_value(&scalar));
    assert!(UiProperty::Radius.accepts_value(&scalar));
    assert_eq!(UiProperty::Opacity.validate_value(&scalar), Ok(()));
}

#[test]
fn property_rejects_mismatched_values() {
    let value = PropertyValue::Size(iced::Size::new(10.0, 20.0));

    assert_eq!(
        UiProperty::Opacity.validate_value(&value),
        Err(PropertyKindError {
            property: UiProperty::Opacity,
            expected: PropertyValueKind::Scalar,
            actual: PropertyValueKind::Size,
        })
    );
}

#[test]
fn property_value_kind_reports_shape() {
    assert_eq!(PropertyValue::Scalar(1.0).kind(), PropertyValueKind::Scalar);
    assert_eq!(
        PropertyValue::Vector2(iced::Vector::new(1.0, 2.0)).kind(),
        PropertyValueKind::Vector2
    );
    assert_eq!(
        PropertyValue::Transform(TransformValue::identity()).kind(),
        PropertyValueKind::Transform
    );
}
