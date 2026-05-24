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
}

#[cfg(test)]
mod tests {
    use super::{UiProperty, UiPropertyCategory};

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
}
