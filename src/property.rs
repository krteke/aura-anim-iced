//! Visual property identifiers, value containers, and composition helpers.

mod kind;
mod spec;
#[cfg(test)]
mod tests;
mod value;

pub use kind::{Color, PropertyValueKind, Rectangle, Scalar, Shadow, Size, Transform, Vector2};
pub use spec::{PropertySpec, RawPropertySpec};
pub use value::{PropertyValue, TransformValue};

/// Built-in opacity property.
pub const OPACITY: PropertySpec<Scalar> =
    PropertySpec::new(PropertyKey::new("aura", "opacity"), 10);
/// Built-in translate property.
pub const TRANSLATE: PropertySpec<Vector2> =
    PropertySpec::new(PropertyKey::new("aura", "translate"), 19);
/// Built-in uniform scale property.
pub const SCALE: PropertySpec<Scalar> = PropertySpec::new(PropertyKey::new("aura", "scale"), 20);
/// Built-in width property.
pub const WIDTH: PropertySpec<Scalar> = PropertySpec::new(PropertyKey::new("aura", "width"), 30);
/// Built-in height property.
pub const HEIGHT: PropertySpec<Scalar> = PropertySpec::new(PropertyKey::new("aura", "height"), 31);
/// Built-in padding property.
pub const PADDING: PropertySpec<Scalar> =
    PropertySpec::new(PropertyKey::new("aura", "padding"), 40);
/// Built-in border radius property.
pub const RADIUS: PropertySpec<Scalar> = PropertySpec::new(PropertyKey::new("aura", "radius"), 50);
/// Built-in background color property.
pub const BACKGROUND: PropertySpec<Color> =
    PropertySpec::new(PropertyKey::new("aura", "background"), 60);
/// Built-in border color property.
pub const BORDER_COLOR: PropertySpec<Color> =
    PropertySpec::new(PropertyKey::new("aura", "border-color"), 70);
/// Built-in text color property.
pub const TEXT_COLOR: PropertySpec<Color> =
    PropertySpec::new(PropertyKey::new("aura", "text-color"), 80);
/// Built-in shadow property.
pub const SHADOW: PropertySpec<Shadow> = PropertySpec::new(PropertyKey::new("aura", "shadow"), 90);

/// A type-erased collection of sampled properties for one animation target.
///
/// Later entries with the same [`RawPropertySpec`] replace earlier entries when
/// snapshots are merged. The runtime uses this as the value passed from
/// keyframes/timelines into target-scoped ticks.
///
/// # Example
///
/// ```
/// use aura_anim_iced::property::{self, PropertySnapshot, PropertyValue};
///
/// let mut base = PropertySnapshot::from((property::OPACITY, 0.25));
/// base.merge(PropertySnapshot::from(vec![
///     (property::OPACITY, 1.0),
///     (property::SCALE, 1.05),
/// ]));
///
/// let opacity = base
///     .find_property(&property::OPACITY.raw())
///     .expect("opacity sample");
///
/// assert_eq!(opacity.value(), &PropertyValue::Scalar(1.0));
/// ```
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
    /// Creates an empty property snapshot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Returns whether the snapshot contains no properties.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the sampled properties in composition order when normalized.
    #[must_use]
    pub fn entries(&self) -> &[PropertyEntry] {
        &self.entries
    }

    pub(crate) fn sort_by_composition_key(&mut self) {
        self.entries
            .sort_by_key(|entry| entry.spec.composition_order());
    }

    /// Merges another snapshot into this one.
    ///
    /// Existing properties with the same raw spec are replaced; new properties
    /// are appended and the result is sorted by composition order.
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

    /// Returns the entry for a raw property spec, if present.
    #[must_use]
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

impl Default for PropertySnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// One sampled property value after type erasure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PropertyEntry {
    spec: RawPropertySpec,
    value: PropertyValue,
}

impl PropertyEntry {
    /// Creates a sampled property entry from a typed property spec.
    #[must_use]
    pub fn new<K: PropertyValueKind>(spec: PropertySpec<K>, value: K::Inner) -> Self {
        Self {
            spec: spec.raw(),
            value: K::wrap(value),
        }
    }

    /// Returns the erased property spec.
    #[must_use]
    pub fn spec(&self) -> &RawPropertySpec {
        &self.spec
    }

    /// Returns the erased property value.
    #[must_use]
    pub fn value(&self) -> &PropertyValue {
        &self.value
    }

    pub(crate) fn new_unchecked(spec: RawPropertySpec, value: PropertyValue) -> Self {
        Self { spec, value }
    }
}

/// Stable property identity.
///
/// The namespace/name pair is used instead of a central enum so downstream code
/// can declare additional properties without modifying this crate.
///
/// # Example
///
/// ```
/// use aura_anim_iced::property::{PropertyKey, PropertySpec, Scalar};
///
/// const TOAST_Y: PropertySpec<Scalar> =
///     PropertySpec::new(PropertyKey::new("app", "toast-y"), 21);
///
/// assert_eq!(TOAST_Y.raw().key().name(), "toast-y");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertyKey {
    namespace: &'static str,
    name: &'static str,
}

impl PropertyKey {
    /// Creates a property key.
    #[must_use]
    pub const fn new(namespace: &'static str, name: &'static str) -> Self {
        Self { namespace, name }
    }

    /// Returns the key namespace.
    #[must_use]
    pub const fn namespace(&self) -> &'static str {
        self.namespace
    }

    /// Returns the key name inside its namespace.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }
}
