use std::{
    ops::{Add, AddAssign},
    time::Duration as StdDuration,
};

use crate::timing::utils::std_duration_from_secs;

/// A non-negative animation duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Duration(StdDuration);

impl Duration {
    /// A zero-length duration.
    pub const ZERO: Self = Self(StdDuration::ZERO);

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Creates a duration from milliseconds.
    #[must_use]
    pub fn from_millis(millis: f64) -> Self {
        Self(std_duration_from_secs(millis / 1000.0))
    }

    /// Creates a duration from seconds.
    #[must_use]
    pub fn from_secs(seconds: f64) -> Self {
        Self(std_duration_from_secs(seconds))
    }

    /// Returns this duration in milliseconds.
    #[must_use]
    pub const fn as_millis(self) -> f64 {
        self.0.as_secs_f64() * 1000.0
    }

    /// Returns this duration in seconds.
    #[must_use]
    pub const fn as_secs(self) -> f64 {
        self.0.as_secs_f64()
    }

    pub(crate) fn checked_mul(self, rhs: u32) -> Option<Self> {
        self.0.checked_mul(rhs).map(Self)
    }

    pub(crate) fn checked_add_delay(self, rhs: Delay) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }

    pub(crate) fn saturating_sub(self, rhs: Self) -> Self {
        Self(self.0.saturating_sub(rhs.0))
    }

    pub(crate) fn min(self, rhs: Self) -> Self {
        Self(self.0.min(rhs.0))
    }

    pub(crate) fn max(self, rhs: Self) -> Self {
        Self(self.0.max(rhs.0))
    }
}

impl AddAssign for Duration {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl From<StdDuration> for Duration {
    fn from(value: StdDuration) -> Self {
        Self(value)
    }
}

/// A non-negative animation start delay.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Delay(StdDuration);

impl Delay {
    /// No start delay.
    pub const ZERO: Self = Self(StdDuration::ZERO);

    /// Creates a delay from milliseconds.
    #[must_use]
    pub fn from_millis(millis: f64) -> Self {
        Self(std_duration_from_secs(millis / 1000.0))
    }

    /// Creates a delay from seconds.
    #[must_use]
    pub fn from_secs(seconds: f64) -> Self {
        Self(std_duration_from_secs(seconds))
    }

    /// Returns this delay in milliseconds.
    #[must_use]
    pub const fn as_millis(self) -> f64 {
        self.0.as_secs_f64() * 1000.0
    }
}

impl From<StdDuration> for Delay {
    fn from(value: StdDuration) -> Self {
        Self(value)
    }
}
