/// A typed value carried by an animation property.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyValue {
    /// A single scalar value.
    Scalar(f32),
    /// An Iced two-dimensional vector value.
    Vector2(iced::Vector),
    /// An Iced size value.
    Size(iced::Size),
    /// An Iced rectangle value.
    Rectangle(iced::Rectangle),
    /// A transform value used by transform-friendly properties.
    Transform(TransformValue),
    /// An Iced color value.
    Color(iced::Color),
    /// An Iced shadow value.
    Shadow(iced::Shadow),
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
