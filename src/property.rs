//! Visual property identifiers, value containers, and composition helpers.

mod kind;
mod spec;
#[cfg(test)]
mod tests;
mod value;

use crate::property::kind::{Color, Scalar, Shadow};

pub use kind::PropertyValueKind;
pub use spec::{PropertySpec, RawPropertySpec};
pub use value::{PropertyValue, TransformValue};

pub const OPACITY: PropertySpec<Scalar> =
    PropertySpec::new(PropertyKey::new("aura", "opacity"), 10);
pub const SCALE: PropertySpec<Scalar> = PropertySpec::new(PropertyKey::new("aura", "scale"), 20);
pub const WIDTH: PropertySpec<Scalar> = PropertySpec::new(PropertyKey::new("aura", "width"), 30);
pub const HEIGHT: PropertySpec<Scalar> = PropertySpec::new(PropertyKey::new("aura", "height"), 31);
pub const PADDING: PropertySpec<Scalar> =
    PropertySpec::new(PropertyKey::new("aura", "padding"), 40);
pub const RADIUS: PropertySpec<Scalar> = PropertySpec::new(PropertyKey::new("aura", "radius"), 50);
pub const BACKGROUND: PropertySpec<Color> =
    PropertySpec::new(PropertyKey::new("aura", "background"), 60);
pub const BORDER_COLOR: PropertySpec<Color> =
    PropertySpec::new(PropertyKey::new("aura", "border-color"), 70);
pub const TEXT_COLOR: PropertySpec<Color> =
    PropertySpec::new(PropertyKey::new("aura", "text-color"), 80);
pub const SHADOW: PropertySpec<Shadow> = PropertySpec::new(PropertyKey::new("aura", "shadow"), 90);

#[derive(Debug, Clone, PartialEq)]
pub struct PropertySnapshot {
    entries: Vec<PropertyEntry>,
}

impl<K: PropertyValueKind> From<Vec<(PropertySpec<K>, K::Inner)>> for PropertySnapshot {
    fn from(value: Vec<(PropertySpec<K>, K::Inner)>) -> Self {
        Self {
            entries: value
                .into_iter()
                .map(|(spec, value)| PropertyEntry::new(spec, value))
                .collect(),
        }
    }
}

impl<K: PropertyValueKind> From<(PropertySpec<K>, K::Inner)> for PropertySnapshot {
    fn from(value: (PropertySpec<K>, K::Inner)) -> Self {
        Self {
            entries: vec![PropertyEntry::new(value.0, value.1)],
        }
    }
}

impl From<Vec<PropertyEntry>> for PropertySnapshot {
    fn from(entries: Vec<PropertyEntry>) -> Self {
        Self { entries }
    }
}

impl PropertySnapshot {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub const fn with_entries(entries: Vec<PropertyEntry>) -> Self {
        Self { entries }
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn entries(&self) -> &[PropertyEntry] {
        &self.entries
    }

    pub fn sort_by_composition_key(&mut self) {
        self.entries
            .sort_by_key(|entry| entry.spec.composition_order());
    }

    pub fn merge(&mut self, other: Self) {
        other.entries.into_iter().for_each(|snapshot| {
            if let Some(entry) = self.find_property_mut(&snapshot.spec) {
                entry.value = snapshot.value;
            } else {
                self.entries.push(snapshot);
            }
        });

        self.sort_by_composition_key();
    }

    pub fn find_property(&self, property: &RawPropertySpec) -> Option<&PropertyEntry> {
        self.entries.iter().find(|entry| entry.spec == *property)
    }

    pub(crate) fn push(&mut self, entry: PropertyEntry) {
        self.entries.push(entry);
    }

    fn find_property_mut(&mut self, property: &RawPropertySpec) -> Option<&mut PropertyEntry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.spec == *property)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyEntry {
    spec: RawPropertySpec,
    value: PropertyValue,
}

impl PropertyEntry {
    pub fn new<K: PropertyValueKind>(spec: PropertySpec<K>, value: K::Inner) -> Self {
        let value = value.into();
        Self {
            spec: spec.raw(),
            value: K::wrap(value),
        }
    }

    pub fn spec(&self) -> &RawPropertySpec {
        &self.spec
    }

    pub fn value(&self) -> &PropertyValue {
        &self.value
    }

    pub(crate) fn set_value(&mut self, value: PropertyValue) -> Self {
        self.value = value;
        *self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyKey {
    namespace: &'static str,
    name: &'static str,
}

impl PropertyKey {
    pub const fn new(namespace: &'static str, name: &'static str) -> Self {
        Self { namespace, name }
    }

    pub const fn namespace(&self) -> &'static str {
        self.namespace
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}
