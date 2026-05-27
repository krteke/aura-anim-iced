//! Visual property identifiers, value containers, and composition helpers.

mod error;
mod iced;
mod order;
#[cfg(test)]
mod tests;
mod value;

pub use error::PropertyKindError;
pub use order::{
    PropertyCompositionKey, sort_properties_by_composition, sort_property_entries_by_composition,
};
pub use value::{PropertyValue, PropertyValueKind, TransformValue};

/// A sampled set of property values ready to compose into an Iced view.
pub type PropertySnapshot = Vec<(UiProperty, PropertyValue)>;

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

    /// Returns the deterministic composition key for this property.
    #[must_use]
    pub const fn composition_key(self) -> PropertyCompositionKey {
        PropertyCompositionKey::new(self.composition_order(), self.id())
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
    pub fn validate_value(self, value: &PropertyValue) -> Result<(), PropertyKindError> {
        let expected = self.expected_value_kind();
        let actual = value.kind();

        if expected.matches(value) {
            Ok(())
        } else {
            Err(PropertyKindError {
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
