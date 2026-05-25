use super::{PropertyValueKind, UiProperty};

/// A typed property/value mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyValueError {
    /// The property being validated.
    pub property: UiProperty,
    /// The expected value kind.
    pub expected: PropertyValueKind,
    /// The actual value kind.
    pub actual: PropertyValueKind,
}
