/// A typed value carried by an animation property.
#[derive(Debug, Clone, PartialEq)]
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
            Self::Color(_) => PropertyValueKind::Color,
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
