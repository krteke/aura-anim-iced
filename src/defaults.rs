//! Product-level animation defaults.

use crate::{
    behavior::BehaviorRule,
    property::{PropertySpec, PropertyValueKind},
    timing::{Duration, FillMode, Timing},
};

#[cfg(feature = "spring")]
mod spring;

#[cfg(feature = "spring")]
pub use spring::SpringMotionDefaults;

/// Product-level defaults for ordinary UI animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DefaultMotions {
    duration: Duration,
    easing: iced::animation::Easing,
    fill_mode: FillMode,
    #[cfg(feature = "spring")]
    spring: SpringMotionDefaults,
}

impl DefaultMotions {
    /// Creates product defaults from core timing settings.
    #[must_use]
    pub const fn new(
        duration: Duration,
        easing: iced::animation::Easing,
        fill_mode: FillMode,
    ) -> Self {
        Self {
            duration,
            easing,
            fill_mode,
            #[cfg(feature = "spring")]
            spring: SpringMotionDefaults::new(Duration::ZERO, 0.0, 0.0),
        }
    }

    /// Returns the default finite duration for product animation.
    #[must_use]
    pub const fn duration(self) -> Duration {
        self.duration
    }

    /// Returns the default easing curve for product animation.
    #[must_use]
    pub const fn easing(self) -> iced::animation::Easing {
        self.easing
    }

    /// Returns the default fill mode for product animation.
    #[must_use]
    pub const fn fill_mode(self) -> FillMode {
        self.fill_mode
    }

    /// Returns the default spring motion settings.
    #[cfg(feature = "spring")]
    #[must_use]
    pub const fn spring(self) -> SpringMotionDefaults {
        self.spring
    }

    /// Replaces the default duration.
    #[must_use]
    pub const fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Replaces the default easing curve.
    #[must_use]
    pub const fn with_easing(mut self, easing: iced::animation::Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Replaces the default fill mode.
    #[must_use]
    pub const fn with_fill_mode(mut self, fill_mode: FillMode) -> Self {
        self.fill_mode = fill_mode;
        self
    }

    /// Replaces the default spring motion settings.
    #[cfg(feature = "spring")]
    #[must_use]
    pub const fn with_spring(mut self, spring: SpringMotionDefaults) -> Self {
        self.spring = spring;
        self
    }

    /// Builds a timing value from product defaults.
    #[must_use]
    pub fn timing(self) -> Timing {
        Timing::new(self.duration.as_millis())
            .with_easing(self.easing)
            .with_fill_mode(self.fill_mode)
    }

    /// Builds a behavior rule using product default timing.
    #[must_use]
    pub fn behavior<K>(self, property: PropertySpec<K>) -> BehaviorRule<K>
    where
        K: PropertyValueKind,
    {
        BehaviorRule::new(property).with_timing(self.timing())
    }
}

impl Default for DefaultMotions {
    fn default() -> Self {
        #[cfg(not(feature = "spring"))]
        {
            Self::new(
                Duration::from_millis(180.0),
                iced::animation::Easing::EaseOut,
                FillMode::Forwards,
            )
        }

        #[cfg(feature = "spring")]
        {
            let mut defaults = Self::new(
                Duration::from_millis(180.0),
                iced::animation::Easing::EaseOut,
                FillMode::Forwards,
            );

            defaults.spring = SpringMotionDefaults::default();

            defaults
        }
    }
}
