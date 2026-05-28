use crate::{PropertyValue, TransformValue};

macro_rules! impl_property_value_kinds {
    (
        $value_enum:ident {
            $(
                $kind:ident => $variant:ident($inner:ty)
            ),* $(,)?
        }
    ) => {
        pub trait PropertyValueKind {
            type Inner;

            fn wrap(value: Self::Inner) -> $value_enum;
        }

        $(
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
