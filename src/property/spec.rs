use std::marker::PhantomData;

use crate::property::{PropertyKey, kind::PropertyValueKind};

/// Type-erased property metadata stored in sampled snapshots.
///
/// A raw spec is the stable identity used for merging and lookup after the
/// typed [`PropertySpec`] has been erased into [`PropertyValue`](crate::property::PropertyValue).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawPropertySpec {
    key: PropertyKey,
    composition_order: u8,
}

impl RawPropertySpec {
    pub(crate) const fn new(key: PropertyKey, composition_order: u8) -> Self {
        Self {
            key,
            composition_order,
        }
    }

    /// Returns the stable property key.
    #[must_use]
    pub const fn key(&self) -> PropertyKey {
        self.key
    }

    /// Returns the ordering bucket used when composing multiple properties.
    #[must_use]
    pub const fn composition_order(&self) -> u8 {
        self.composition_order
    }
}

/// Typed property metadata.
///
/// The generic marker controls which value type can be inserted for this
/// property, while [`raw`](Self::raw) exposes the erased identity used by the
/// runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertySpec<K: PropertyValueKind> {
    raw: RawPropertySpec,
    _kind: PhantomData<fn() -> K>,
}

impl<K: PropertyValueKind> PropertySpec<K> {
    /// Creates a typed property declaration.
    ///
    /// `composition_order` is intentionally independent from the string key so
    /// third-party property families can add stable ordering without extending a
    /// central enum.
    #[must_use]
    pub const fn new(key: PropertyKey, composition_order: u8) -> Self {
        Self {
            raw: RawPropertySpec::new(key, composition_order),
            _kind: PhantomData,
        }
    }

    /// Returns the erased property metadata.
    #[must_use]
    pub const fn raw(self) -> RawPropertySpec {
        self.raw
    }
}
