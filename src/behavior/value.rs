use crate::property::{
    Color, PropertyValue, PropertyValueKind, Rectangle, Scalar, Shadow, Size, Transform, Vector2,
};

/// Property marker support for value-change transition sampling.
///
/// Custom property marker types can implement this trait when their erased
/// [`PropertyValue`] variant can be converted back into the marker's inner
/// value.
pub trait TransitionValueKind: PropertyValueKind {
    /// Extracts this marker's concrete value from an erased property value.
    fn unwrap_transition_value(value: &PropertyValue) -> Option<Self::Inner>;
}

macro_rules! impl_transition_value_kind {
    ($($kind:ident), *) => {
        $(
            impl TransitionValueKind for $kind {
                fn unwrap_transition_value(value: &PropertyValue) -> Option<Self::Inner> {
                    let PropertyValue::$kind(value) = value else {
                        return None;
                    };

                    Some(*value)
                }
            }
        )*
    };
}

impl_transition_value_kind!(Scalar, Vector2, Size, Rectangle, Transform, Color, Shadow);
