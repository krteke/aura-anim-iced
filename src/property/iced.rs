use super::PropertyValue;

impl From<iced::Point> for PropertyValue {
    fn from(value: iced::Point) -> Self {
        Self::Vector2(iced::Vector::new(value.x, value.y))
    }
}

impl From<iced::Vector> for PropertyValue {
    fn from(value: iced::Vector) -> Self {
        Self::Vector2(value)
    }
}

impl From<iced::Size> for PropertyValue {
    fn from(value: iced::Size) -> Self {
        Self::Size(value)
    }
}

impl From<iced::Rectangle> for PropertyValue {
    fn from(value: iced::Rectangle) -> Self {
        Self::Rectangle(value)
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
