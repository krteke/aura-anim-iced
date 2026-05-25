use super::{PropertyValue, RectangleValue, SizeValue, Vector2Value};

impl From<iced::Point> for PropertyValue {
    fn from(value: iced::Point) -> Self {
        Self::Vector2(Vector2Value::new(value.x, value.y))
    }
}

impl From<iced::Vector> for PropertyValue {
    fn from(value: iced::Vector) -> Self {
        Self::Vector2(Vector2Value::new(value.x, value.y))
    }
}

impl From<iced::Size> for PropertyValue {
    fn from(value: iced::Size) -> Self {
        Self::Size(SizeValue::new(value.width, value.height))
    }
}

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

impl From<iced::Color> for PropertyValue {
    fn from(value: iced::Color) -> Self {
        Self::Color(value)
    }
}

impl From<iced::Shadow> for PropertyValue {
    fn from(value: iced::Shadow) -> Self {
        Self::Shadow(value)
    }
}
