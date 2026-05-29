use crate::{
    property::{self, PropertySnapshot, PropertyValue},
    runtime::{AnimationTargetId, AnimationTick},
};

/// View-friendly effects extracted from sampled animation properties.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EffectSnapshot {
    /// Element opacity.
    pub opacity: Option<f32>,
    /// Element width.
    pub width: Option<f32>,
    /// Element height.
    pub height: Option<f32>,
    /// Element padding.
    pub padding: Option<f32>,
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

        for entry in properties.entries() {
            match entry.value() {
                PropertyValue::Scalar(value) if *entry.spec() == property::OPACITY.raw() => {
                    effects.opacity = Some(*value);
                }
                PropertyValue::Scalar(value) if *entry.spec() == property::WIDTH.raw() => {
                    effects.width = Some(*value);
                }
                PropertyValue::Scalar(value) if *entry.spec() == property::HEIGHT.raw() => {
                    effects.height = Some(*value);
                }
                PropertyValue::Scalar(value) if *entry.spec() == property::PADDING.raw() => {
                    effects.padding = Some(*value);
                }
                PropertyValue::Scalar(value) if *entry.spec() == property::SCALE.raw() => {
                    effects.scale = Some(*value);
                }
                PropertyValue::Scalar(value) if *entry.spec() == property::RADIUS.raw() => {
                    effects.radius = Some(*value);
                }
                PropertyValue::Color(value) if *entry.spec() == property::BACKGROUND.raw() => {
                    effects.background = Some(*value);
                }
                PropertyValue::Color(value) if *entry.spec() == property::BORDER_COLOR.raw() => {
                    effects.border_color = Some(*value);
                }
                PropertyValue::Color(value) if *entry.spec() == property::TEXT_COLOR.raw() => {
                    effects.text_color = Some(*value);
                }
                PropertyValue::Shadow(value) if *entry.spec() == property::SHADOW.raw() => {
                    effects.shadow = Some(*value);
                }
                _ => {}
            }
        }

        effects
    }

    /// Extracts view-friendly effects from a runtime tick for one target.
    #[must_use]
    pub fn from_tick_for(tick: &AnimationTick, target: AnimationTargetId) -> Self {
        tick.properties_for(target)
            .map(Self::from_properties)
            .unwrap_or_default()
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

/// Extracts view-friendly effects from a runtime tick for one target.
#[must_use]
pub fn tick_effect_snapshot_for(tick: &AnimationTick, target: AnimationTargetId) -> EffectSnapshot {
    EffectSnapshot::from_tick_for(tick, target)
}
