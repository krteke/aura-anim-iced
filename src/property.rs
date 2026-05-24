#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UiProperty {
    Opacity,
    TranslateX,
    TranslateY,
    Scale,
    Rotate,
    Width,
    Height,
    Padding,
    Radius,
    Background,
    BorderColor,
    TextColor,
    Shadow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Scalar(f32),
}
