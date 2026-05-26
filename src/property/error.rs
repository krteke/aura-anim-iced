use std::{error::Error, fmt::Display};

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

impl Display for PropertyValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Property value error: expected {}, got {}",
            self.expected, self.actual
        )
    }
}

impl Error for PropertyValueError {}
