//! Visual property identifiers and value containers.

/// A stable visual property that can be animated and sampled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiProperty {
    /// Element opacity.
    Opacity,
    /// Horizontal translation.
    TranslateX,
    /// Vertical translation.
    TranslateY,
    /// Uniform scale.
    Scale,
    /// Rotation angle.
    Rotate,
    /// Element width.
    Width,
    /// Element height.
    Height,
    /// Element padding.
    Padding,
    /// Corner radius.
    Radius,
    /// Background color.
    Background,
    /// Border color.
    BorderColor,
    /// Text color.
    TextColor,
    /// Shadow style.
    Shadow,
}

impl UiProperty {
    /// All v0.1 visual properties in stable ID order.
    pub const ALL: [Self; 13] = [
        Self::Opacity,
        Self::TranslateX,
        Self::TranslateY,
        Self::Scale,
        Self::Rotate,
        Self::Width,
        Self::Height,
        Self::Padding,
        Self::Radius,
        Self::Background,
        Self::BorderColor,
        Self::TextColor,
        Self::Shadow,
    ];

    /// Returns the stable numeric ID for serialized tracks and diagnostics.
    #[must_use]
    pub const fn id(self) -> u16 {
        match self {
            Self::Opacity => 1,
            Self::TranslateX => 2,
            Self::TranslateY => 3,
            Self::Scale => 4,
            Self::Rotate => 5,
            Self::Width => 6,
            Self::Height => 7,
            Self::Padding => 8,
            Self::Radius => 9,
            Self::Background => 10,
            Self::BorderColor => 11,
            Self::TextColor => 12,
            Self::Shadow => 13,
        }
    }

    /// Returns the broad visual category for this property.
    #[must_use]
    pub const fn category(self) -> UiPropertyCategory {
        match self {
            Self::Opacity => UiPropertyCategory::Opacity,
            Self::TranslateX | Self::TranslateY | Self::Scale | Self::Rotate => {
                UiPropertyCategory::Transform
            }
            Self::Width | Self::Height | Self::Padding => UiPropertyCategory::Size,
            Self::Radius => UiPropertyCategory::Radius,
            Self::Background | Self::BorderColor | Self::TextColor => UiPropertyCategory::Color,
            Self::Shadow => UiPropertyCategory::Shadow,
        }
    }

    /// Returns the default composition order used when applying snapshots.
    #[must_use]
    pub const fn composition_order(self) -> u8 {
        match self {
            Self::Opacity => 10,
            Self::TranslateX | Self::TranslateY | Self::Scale | Self::Rotate => 20,
            Self::Width | Self::Height => 30,
            Self::Padding => 40,
            Self::Radius => 50,
            Self::Background => 60,
            Self::BorderColor => 70,
            Self::TextColor => 80,
            Self::Shadow => 90,
        }
    }

    /// Returns the expected value kind for this property.
    #[must_use]
    pub const fn expected_value_kind(self) -> PropertyValueKind {
        match self {
            Self::Opacity
            | Self::TranslateX
            | Self::TranslateY
            | Self::Scale
            | Self::Rotate
            | Self::Width
            | Self::Height
            | Self::Padding
            | Self::Radius => PropertyValueKind::Scalar,
            Self::Background | Self::BorderColor | Self::TextColor => PropertyValueKind::Color,
            Self::Shadow => PropertyValueKind::Shadow,
        }
    }

    /// Returns whether `value` can be used for this property.
    #[must_use]
    pub fn accepts_value(self, value: &PropertyValue) -> bool {
        self.expected_value_kind().matches(value)
    }

    /// Validates that `value` can be used for this property.
    pub fn validate_value(self, value: &PropertyValue) -> Result<(), PropertyValueError> {
        let expected = self.expected_value_kind();
        let actual = value.kind();

        if expected.matches(value) {
            Ok(())
        } else {
            Err(PropertyValueError {
                property: self,
                expected,
                actual,
            })
        }
    }
}

/// Broad visual property categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiPropertyCategory {
    /// Opacity properties.
    Opacity,
    /// Transform properties.
    Transform,
    /// Size and spacing properties.
    Size,
    /// Radius properties.
    Radius,
    /// Color properties.
    Color,
    /// Shadow properties.
    Shadow,
}

/// A typed value carried by an animation property.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// A single scalar value.
    Scalar(f32),
    /// A two-dimensional vector-like value.
    Vector2(Vector2Value),
    /// A width and height value.
    Size(SizeValue),
    /// A rectangle value.
    Rectangle(RectangleValue),
    /// A transform value used by transform-friendly properties.
    Transform(TransformValue),
    /// An Iced color value.
    #[cfg(feature = "iced")]
    Color(iced::Color),
    /// An Iced shadow value.
    #[cfg(feature = "iced")]
    Shadow(iced::Shadow),
}

impl PropertyValue {
    /// Returns the kind represented by this value.
    #[must_use]
    pub const fn kind(&self) -> PropertyValueKind {
        match self {
            Self::Scalar(_) => PropertyValueKind::Scalar,
            Self::Vector2(_) => PropertyValueKind::Vector2,
            Self::Size(_) => PropertyValueKind::Size,
            Self::Rectangle(_) => PropertyValueKind::Rectangle,
            Self::Transform(_) => PropertyValueKind::Transform,
            #[cfg(feature = "iced")]
            Self::Color(_) => PropertyValueKind::Color,
            #[cfg(feature = "iced")]
            Self::Shadow(_) => PropertyValueKind::Shadow,
        }
    }
}

/// The high-level kind of a property value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PropertyValueKind {
    /// Scalar values.
    Scalar,
    /// Two-dimensional vector values.
    Vector2,
    /// Size values.
    Size,
    /// Rectangle values.
    Rectangle,
    /// Transform values.
    Transform,
    /// Color values.
    Color,
    /// Shadow values.
    Shadow,
}

impl PropertyValueKind {
    /// Returns whether this kind matches `value`.
    #[must_use]
    pub fn matches(self, value: &PropertyValue) -> bool {
        self == value.kind()
    }
}

/// A typed property/value mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyValueError {
    /// The property being validated.
    pub property: UiProperty,
    /// The expected value kind.
    pub expected: PropertyValueKind,
    /// The actual value kind.
    pub actual: PropertyValueKind,
}

/// A two-dimensional value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2Value {
    /// Horizontal component.
    pub x: f32,
    /// Vertical component.
    pub y: f32,
}

impl Vector2Value {
    /// Creates a two-dimensional value.
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// A size value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeValue {
    /// Width component.
    pub width: f32,
    /// Height component.
    pub height: f32,
}

impl SizeValue {
    /// Creates a size value.
    #[must_use]
    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// A rectangle value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectangleValue {
    /// Horizontal origin.
    pub x: f32,
    /// Vertical origin.
    pub y: f32,
    /// Width component.
    pub width: f32,
    /// Height component.
    pub height: f32,
}

impl RectangleValue {
    /// Creates a rectangle value.
    #[must_use]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// A transform-friendly value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformValue {
    /// Horizontal translation.
    pub translate_x: f32,
    /// Vertical translation.
    pub translate_y: f32,
    /// Uniform scale.
    pub scale: f32,
    /// Rotation angle.
    pub rotate: f32,
}

impl TransformValue {
    /// Creates a transform value.
    #[must_use]
    pub const fn new(translate_x: f32, translate_y: f32, scale: f32, rotate: f32) -> Self {
        Self {
            translate_x,
            translate_y,
            scale,
            rotate,
        }
    }

    /// Returns an identity transform value.
    #[must_use]
    pub const fn identity() -> Self {
        Self::new(0.0, 0.0, 1.0, 0.0)
    }
}

#[cfg(feature = "iced")]
impl From<iced::Point> for PropertyValue {
    fn from(value: iced::Point) -> Self {
        Self::Vector2(Vector2Value::new(value.x, value.y))
    }
}

#[cfg(feature = "iced")]
impl From<iced::Vector> for PropertyValue {
    fn from(value: iced::Vector) -> Self {
        Self::Vector2(Vector2Value::new(value.x, value.y))
    }
}

#[cfg(feature = "iced")]
impl From<iced::Size> for PropertyValue {
    fn from(value: iced::Size) -> Self {
        Self::Size(SizeValue::new(value.width, value.height))
    }
}

#[cfg(feature = "iced")]
impl From<iced::Rectangle> for PropertyValue {
    fn from(value: iced::Rectangle) -> Self {
        Self::Rectangle(RectangleValue::new(
            value.x,
            value.y,
            value.width,
            value.height,
        ))
    }
}

#[cfg(feature = "iced")]
impl From<iced::Color> for PropertyValue {
    fn from(value: iced::Color) -> Self {
        Self::Color(value)
    }
}

#[cfg(feature = "iced")]
impl From<iced::Shadow> for PropertyValue {
    fn from(value: iced::Shadow) -> Self {
        Self::Shadow(value)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        PropertyValue, PropertyValueError, PropertyValueKind, RectangleValue, SizeValue,
        TransformValue, UiProperty, UiPropertyCategory, Vector2Value,
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
    fn property_composition_order_is_monotonic_by_groups() {
        let mut properties = UiProperty::ALL;
        properties.sort_by_key(|property| property.composition_order());

        assert_eq!(properties.first(), Some(&UiProperty::Opacity));
        assert_eq!(properties.last(), Some(&UiProperty::Shadow));
        assert!(
            UiProperty::Background.composition_order()
                < UiProperty::BorderColor.composition_order()
        );
        assert!(
            UiProperty::BorderColor.composition_order() < UiProperty::TextColor.composition_order()
        );
    }

    #[test]
    fn property_values_cover_core_value_shapes() {
        assert_eq!(PropertyValue::Scalar(0.5), PropertyValue::Scalar(0.5));
        assert_eq!(
            PropertyValue::Vector2(Vector2Value::new(1.0, 2.0)),
            PropertyValue::Vector2(Vector2Value { x: 1.0, y: 2.0 })
        );
        assert_eq!(
            PropertyValue::Size(SizeValue::new(10.0, 20.0)),
            PropertyValue::Size(SizeValue {
                width: 10.0,
                height: 20.0,
            })
        );
        assert_eq!(
            PropertyValue::Rectangle(RectangleValue::new(1.0, 2.0, 3.0, 4.0)),
            PropertyValue::Rectangle(RectangleValue {
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
        let value = PropertyValue::Size(SizeValue::new(10.0, 20.0));

        assert_eq!(
            UiProperty::Opacity.validate_value(&value),
            Err(PropertyValueError {
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
            PropertyValue::Vector2(Vector2Value::new(1.0, 2.0)).kind(),
            PropertyValueKind::Vector2
        );
        assert_eq!(
            PropertyValue::Transform(TransformValue::identity()).kind(),
            PropertyValueKind::Transform
        );
    }
}
