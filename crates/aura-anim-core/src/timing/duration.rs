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

    /// Returns whether this duration is zero.
    #[must_use]
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

    pub(crate) fn checked_sub_delay(self, rhs: Delay) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }

    pub(crate) fn divided_by(self, divisor: f64) -> Self {
        if divisor.is_finite() && divisor > 0.0 {
            let seconds = self.0.as_secs_f64() / divisor;
            if seconds.is_infinite() {
                Self(StdDuration::MAX)
            } else {
                Self(std_duration_from_secs(seconds))
            }
        } else {
            self
        }
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

#[cfg(test)]
mod tests {
    use super::{Delay, Duration};
    use float_cmp::assert_approx_eq;

    #[test]
    fn duration_constructors_sanitize_negative_and_nan_values() {
        assert!(Duration::from_millis(-10.0).is_zero());
        assert!(Duration::from_secs(f64::NAN).is_zero());
        assert!(Delay::from_millis(-10.0).as_millis().abs() <= f64::EPSILON);
    }

    #[test]
    fn duration_reports_seconds_and_milliseconds() {
        let duration = Duration::from_millis(1_250.0);

        assert_approx_eq!(f64, duration.as_secs(), 1.25);
        assert_approx_eq!(f64, duration.as_millis(), 1_250.0);
    }

    #[test]
    fn duration_arithmetic_is_saturating_where_expected() {
        let short = Duration::from_millis(25.0);
        let long = Duration::from_millis(100.0);

        assert_eq!(short.saturating_sub(long), Duration::ZERO);
        assert_eq!(short.min(long), short);
        assert_eq!(short.max(long), long);
        assert_approx_eq!(f64, short.checked_mul(2).unwrap().as_millis(), 50.0);
        assert_approx_eq!(
            f64,
            short
                .checked_add_delay(Delay::from_millis(25.0))
                .unwrap()
                .as_millis(),
            50.0
        );
        assert_eq!(
            long.checked_sub_delay(Delay::from_millis(25.0)),
            Some(Duration::from_millis(75.0))
        );
        assert_eq!(short.checked_sub_delay(Delay::from_millis(50.0)), None);
        assert_eq!(long.divided_by(2.0), Duration::from_millis(50.0));
        assert_eq!(long.divided_by(0.5), Duration::from_millis(200.0));
        assert_eq!(long.divided_by(0.0), long);
        assert_eq!(long.divided_by(f64::NAN), long);
        assert_eq!(
            long.divided_by(f64::MIN_POSITIVE),
            Duration(std::time::Duration::MAX)
        );
    }
}
