use crate::{
    AnimationTargetId, AnimationTick, EffectSnapshot, PropertySnapshot, PropertySpec,
    PropertyValueRead,
};

/// View-facing animation output for one target.
///
/// This helper keeps product view code from repeatedly looking up raw property
/// entries and matching erased [`PropertyValue`](crate::PropertyValue) variants.
#[derive(Debug, Clone, Copy)]
pub struct AnimationTargetOutput<'a> {
    properties: Option<&'a PropertySnapshot>,
}

impl<'a> AnimationTargetOutput<'a> {
    /// Creates target output from optional sampled properties.
    #[must_use]
    pub const fn new(properties: Option<&'a PropertySnapshot>) -> Self {
        Self { properties }
    }

    /// Extracts target output from a runtime tick.
    #[must_use]
    pub fn from_tick(tick: &'a AnimationTick, target: AnimationTargetId) -> Self {
        Self::new(tick.properties_for(target))
    }

    /// Returns whether this target has no current visual output.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.properties.is_none_or(PropertySnapshot::is_empty)
    }

    /// Returns the raw property snapshot for this target.
    #[must_use]
    pub const fn properties(self) -> Option<&'a PropertySnapshot> {
        self.properties
    }

    /// Reads a typed property value for this target.
    #[must_use]
    pub fn get<K>(self, spec: PropertySpec<K>) -> Option<K::Inner>
    where
        K: PropertyValueRead,
    {
        let entry = self.properties?.find_property(&spec.raw())?;

        K::read(entry.value())
    }

    /// Extracts view-friendly built-in effects for this target.
    #[must_use]
    pub fn effects(self) -> EffectSnapshot {
        self.properties
            .map(EffectSnapshot::from_properties)
            .unwrap_or_default()
    }
}

/// Extracts view-facing output for one target from a runtime tick.
#[must_use]
pub fn target_output_for(
    tick: &AnimationTick,
    target: AnimationTargetId,
) -> AnimationTargetOutput<'_> {
    AnimationTargetOutput::from_tick(tick, target)
}
