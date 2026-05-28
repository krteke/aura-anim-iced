use std::marker::PhantomData;

use crate::property::{PropertyKey, kind::PropertyValueKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawPropertySpec {
    key: PropertyKey,
    composition_order: u8,
}

impl RawPropertySpec {
    const fn new(key: PropertyKey, composition_order: u8) -> Self {
        Self {
            key,
            composition_order,
        }
    }

    pub fn composition_order(&self) -> u8 {
        self.composition_order
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PropertySpec<K: PropertyValueKind> {
    raw: RawPropertySpec,
    _kind: PhantomData<fn() -> K>,
}

impl<K: PropertyValueKind> PropertySpec<K> {
    pub const fn new(key: PropertyKey, composition_order: u8) -> Self {
        Self {
            raw: RawPropertySpec::new(key, composition_order),
            _kind: PhantomData,
        }
    }

    pub const fn raw(&self) -> RawPropertySpec {
        self.raw
    }

    pub const fn new_raw(key: PropertyKey, composition_order: u8) -> RawPropertySpec {
        Self::new(key, composition_order).raw
    }
}
