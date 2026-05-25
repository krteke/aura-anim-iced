use super::{PropertyValue, UiProperty};

/// A stable key used to order property composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PropertyCompositionKey {
    /// The broad visual layer order.
    pub order: u8,
    /// The stable property ID used as a tie-breaker inside a layer.
    pub id: u16,
}

impl PropertyCompositionKey {
    /// Creates a composition key.
    #[must_use]
    pub const fn new(order: u8, id: u16) -> Self {
        Self { order, id }
    }
}

/// Sorts properties by deterministic visual composition order.
pub fn sort_properties_by_composition(properties: &mut [UiProperty]) {
    properties.sort_by_key(|property| property.composition_key());
}

/// Sorts property/value entries by deterministic visual composition order.
pub fn sort_property_entries_by_composition(entries: &mut [(UiProperty, PropertyValue)]) {
    entries.sort_by_key(|(property, _)| property.composition_key());
}
