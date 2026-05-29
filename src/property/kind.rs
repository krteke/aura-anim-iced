use crate::{PropertyValue, TransformValue};

macro_rules! impl_property_value_kinds {
    (
        $value_enum:ident {
            $(
                $kind:ident => $variant:ident($inner:ty)
            ),* $(,)?
        }
    ) => {
        /// Type-level marker for values accepted by a [`PropertySpec`](crate::property::PropertySpec).
        ///
        /// The marker keeps property declarations typed while runtime snapshots still carry
        /// values in the erased [`PropertyValue`] container.
        pub trait PropertyValueKind {
            /// Concrete Rust value accepted by this property kind.
            type Inner;

            /// Wraps the concrete value into the erased runtime value container.
            fn wrap(value: Self::Inner) -> $value_enum;
        }

        $(
            #[doc = concat!("Marker type for [`PropertyValue::", stringify!($variant), "`] properties.")]
            #[derive(Debug, Clone, Copy, PartialEq)]
            pub struct $kind;

            impl PropertyValueKind for $kind {
                type Inner = $inner;

                fn wrap(value: Self::Inner) -> $value_enum {
                    $value_enum::$variant(value)
                }
            }
        )*
    };
}

impl_property_value_kinds!(
    PropertyValue {
        Scalar => Scalar(f32),
        Vector2 => Vector2(iced::Vector),
        Size => Size(iced::Size),
        Rectangle => Rectangle(iced::Rectangle),
        Transform => Transform(TransformValue),
        Color => Color(iced::Color),
        Shadow => Shadow(iced::Shadow),
    }
);
