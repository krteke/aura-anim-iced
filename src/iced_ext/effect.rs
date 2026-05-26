use crate::{
    property::{PropertySnapshot, PropertyValue, UiProperty},
    runtime::AnimationTick,
};

/// View-friendly effects extracted from sampled animation properties.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectSnapshot {
    /// Element opacity.
    pub opacity: Option<f32>,
    /// Element translation.
    pub translation: Option<iced::Vector>,
    /// Element scale.
    pub scale: Option<f32>,
    /// Element radius.
    pub radius: Option<f32>,
    /// Element background color.
    pub background: Option<iced::Color>,
    /// Element border color.
    pub border_color: Option<iced::Color>,
    /// Element text color.
    pub text_color: Option<iced::Color>,
    /// Element shadow.
    pub shadow: Option<iced::Shadow>,
}

impl EffectSnapshot {
    /// Extracts view-friendly effects from sampled properties.
    #[must_use]
    pub fn from_properties(properties: &PropertySnapshot) -> Self {
        let mut effects = Self::default();
        let mut translate_x = None;
        let mut translate_y = None;

        for (property, value) in properties {
            match (property, value) {
                (UiProperty::Opacity, PropertyValue::Scalar(value)) => {
                    effects.opacity = Some(*value);
                }
                (UiProperty::TranslateX, PropertyValue::Scalar(value)) => {
                    translate_x = Some(*value);
                }
                (UiProperty::TranslateY, PropertyValue::Scalar(value)) => {
                    translate_y = Some(*value);
                }
                (UiProperty::Scale, PropertyValue::Scalar(value)) => {
                    effects.scale = Some(*value);
                }
                (UiProperty::Radius, PropertyValue::Scalar(value)) => {
                    effects.radius = Some(*value);
                }
                (UiProperty::Background, PropertyValue::Color(value)) => {
                    effects.background = Some(*value);
                }
                (UiProperty::BorderColor, PropertyValue::Color(value)) => {
                    effects.border_color = Some(*value);
                }
                (UiProperty::TextColor, PropertyValue::Color(value)) => {
                    effects.text_color = Some(*value);
                }
                (UiProperty::Shadow, PropertyValue::Shadow(value)) => {
                    effects.shadow = Some(*value);
                }
                _ => {}
            }
        }

        if translate_x.is_some() || translate_y.is_some() {
            effects.translation = Some(iced::Vector::new(
                translate_x.unwrap_or_default(),
                translate_y.unwrap_or_default(),
            ));
        }

        effects
    }

    /// Extracts view-friendly effects from a runtime tick.
    #[must_use]
    pub fn from_tick(tick: &AnimationTick) -> Self {
        Self::from_properties(tick.properties())
    }

    /// Returns whether no effect was extracted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

/// Extracts view-friendly effects from sampled properties.
#[must_use]
pub fn effect_snapshot(properties: &PropertySnapshot) -> EffectSnapshot {
    EffectSnapshot::from_properties(properties)
}

/// Extracts view-friendly effects from a runtime tick.
#[must_use]
pub fn tick_effect_snapshot(tick: &AnimationTick) -> EffectSnapshot {
    EffectSnapshot::from_tick(tick)
}
