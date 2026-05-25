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
