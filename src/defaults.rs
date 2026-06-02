//! Product-level animation defaults.

use crate::{
    behavior::BehaviorRule,
    property::{PropertySpec, PropertyValueKind},
    timing::{Duration, FillMode, Timing},
};

/// Color interpolation strategy used by product-level animation defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorInterpolationMode {
    /// Interpolate color components directly in the current Iced color space.
    #[default]
    Srgb,
}

/// Product spring motion defaults.
///
/// These values describe the default spring feel used by future spring-driven
/// product motion. They are kept separate from runtime sampling so applications
/// can establish consistent motion settings before spring animation sources are
/// added.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringMotionDefaults {
    response: Duration,
    damping_ratio: f32,
    settle_epsilon: f32,
}

impl SpringMotionDefaults {
    /// Creates spring motion defaults.
    #[must_use]
    pub const fn new(response: Duration, damping_ratio: f32, settle_epsilon: f32) -> Self {
        Self {
            response,
            damping_ratio,
            settle_epsilon,
        }
    }

    /// Returns the spring response duration.
    #[must_use]
    pub const fn response(self) -> Duration {
        self.response
    }

    /// Returns the spring damping ratio.
    #[must_use]
    pub const fn damping_ratio(self) -> f32 {
        self.damping_ratio
    }

    /// Returns the settle epsilon used by spring completion checks.
    #[must_use]
    pub const fn settle_epsilon(self) -> f32 {
        self.settle_epsilon
    }
}

impl Default for SpringMotionDefaults {
    fn default() -> Self {
        Self::new(Duration::from_millis(280.0), 0.82, 0.001)
    }
}

/// Product-level defaults for ordinary UI animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DefaultMotions {
    duration: Duration,
    easing: iced::animation::Easing,
    fill_mode: FillMode,
    color_interpolation: ColorInterpolationMode,
    spring: SpringMotionDefaults,
}

impl DefaultMotions {
    /// Creates product defaults from all supported settings.
    #[must_use]
    pub const fn new(
        duration: Duration,
        easing: iced::animation::Easing,
        fill_mode: FillMode,
        color_interpolation: ColorInterpolationMode,
        spring: SpringMotionDefaults,
    ) -> Self {
        Self {
            duration,
            easing,
            fill_mode,
            color_interpolation,
            spring,
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

    /// Returns the default color interpolation mode.
    #[must_use]
    pub const fn color_interpolation(self) -> ColorInterpolationMode {
        self.color_interpolation
    }

    /// Returns the default spring motion settings.
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

    /// Replaces the default color interpolation mode.
    #[must_use]
    pub const fn with_color_interpolation(
        mut self,
        color_interpolation: ColorInterpolationMode,
    ) -> Self {
        self.color_interpolation = color_interpolation;
        self
    }

    /// Replaces the default spring motion settings.
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
        Self::new(
            Duration::from_millis(180.0),
            iced::animation::Easing::EaseOut,
            FillMode::Forwards,
            ColorInterpolationMode::default(),
            SpringMotionDefaults::default(),
        )
    }
}
