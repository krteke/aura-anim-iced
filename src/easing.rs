//! Easing curve definitions and sampling helpers.

const BEZIER_ITERATIONS: usize = 16;

/// Describes how normalized animation progress is transformed before sampling.
///
/// # Examples
///
/// Sample a built-in linear easing curve:
///
/// ```
/// use aura_anim_iced::easing::Easing;
///
/// assert_eq!(Easing::LINEAR.sample(0.5), 0.5);
/// ```
///
/// Build a standard curve from a curve family and mode:
///
/// ```
/// use aura_anim_iced::easing::{Easing, EasingCurve, EasingMode};
///
/// let easing = Easing::standard(EasingCurve::Sine, EasingMode::InOut);
/// let value = easing.sample(0.5);
///
/// assert!((0.49..=0.51).contains(&value));
/// ```
///
/// Use a Material-style preset:
///
/// ```
/// use aura_anim_iced::easing::Easing;
///
/// let value = Easing::MATERIAL_STANDARD.sample(0.5);
///
/// assert!((0.0..=1.0).contains(&value));
/// ```
///
/// # Performance
///
/// Sampling a [`Easing::Linear`] or [`Easing::Standard`] curve performs a fixed
/// amount of scalar floating-point work and does not allocate. Sampling
/// [`Easing::CubicBezier`] also does not allocate, but it performs a fixed
/// binary search over the curve parameter, controlled by `BEZIER_ITERATIONS`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// A constant-rate easing curve.
    Linear,
    /// A built-in easing curve.
    Standard {
        /// The easing curve to use.
        curve: EasingCurve,
        /// The easing mode to use.
        mode: EasingMode,
    },
    /// A cubic Bezier curve from `(0, 0)` to `(1, 1)`.
    CubicBezier(CubicBezier),
}

/// Easing curves for use with [`Easing::Standard`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EasingCurve {
    /// A quadratic easing curve.
    Quadratic,
    /// A cubic easing curve.
    Cubic,
    /// A sine easing curve.
    Sine,
    /// A circular easing curve.
    Circ,
    /// An exponential easing curve.
    Expo,
}

impl EasingCurve {
    fn sample_in(self, progress: f32) -> f32 {
        match self {
            Self::Quadratic => progress.powi(2),
            Self::Cubic => progress.powi(3),
            Self::Sine => 1.0 - (progress * core::f32::consts::FRAC_PI_2).cos(),
            Self::Circ => 1.0 - (1.0 - progress.powi(2)).sqrt(),
            Self::Expo => {
                if progress == 0.0 {
                    0.0
                } else {
                    2.0_f32.powf(10.0 * progress - 10.0)
                }
            }
        }
    }
}

/// Easing modes for use with [`Easing::Standard`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EasingMode {
    /// `In` mode: the easing curve starts at 0 and ends at 1.
    In,
    /// `Out` mode: the easing curve starts at 1 and ends at 0.
    Out,
    /// `InOut` mode: the easing curve starts at 0 and ends at 1, but is symmetrical.
    InOut,
}

impl EasingMode {
    fn sample(self, curve: EasingCurve, progress: f32) -> f32 {
        match self {
            Self::In => curve.sample_in(progress),
            Self::Out => 1.0 - curve.sample_in(1.0 - progress),
            Self::InOut => {
                if progress < 0.5 {
                    0.5 * curve.sample_in(progress * 2.0)
                } else {
                    1.0 - 0.5 * curve.sample_in((1.0 - progress) * 2.0)
                }
            }
        }
    }
}

/// A cubic bezier curve for use with [`Easing::CubicBezier`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    /// The x-coordinate of the first control point.
    pub x1: f32,
    /// The y-coordinate of the first control point.
    pub y1: f32,
    /// The x-coordinate of the second control point.
    pub x2: f32,
    /// The y-coordinate of the second control point.
    pub y2: f32,
}

impl CubicBezier {
    fn sample(self, progress: f32) -> f32 {
        match progress {
            0.0 => return 0.0,
            1.0 => return 1.0,
            _ => {}
        }

        let x1 = self.x1.clamp(0.0, 1.0);
        let x2 = self.x2.clamp(0.0, 1.0);
        let mut lower = 0.0;
        let mut upper = 1.0;
        let mut parameter = progress;

        // TODO: use a more efficient algorithm than brute force iteration
        for _ in 0..BEZIER_ITERATIONS {
            let x = cubic_axis(parameter, x1, x2);

            if x < progress {
                lower = parameter;
            } else {
                upper = parameter;
            }

            parameter = lower.midpoint(upper);
        }

        cubic_axis(parameter, self.y1, self.y2)
    }
}

impl Easing {
    /// Material-style standard easing for balanced UI movement.
    pub const MATERIAL_STANDARD: Self = Self::CubicBezier(CubicBezier {
        x1: 0.2,
        y1: 0.0,
        x2: 0.0,
        y2: 1.0,
    });

    /// Material-style acceleration easing for elements leaving the screen.
    pub const MATERIAL_STANDARD_ACCELERATE: Self = Self::CubicBezier(CubicBezier {
        x1: 0.3,
        y1: 0.0,
        x2: 1.0,
        y2: 1.0,
    });

    /// Material-style deceleration easing for elements entering the screen.
    pub const MATERIAL_STANDARD_DECELERATE: Self = Self::CubicBezier(CubicBezier {
        x1: 0.0,
        y1: 0.0,
        x2: 0.0,
        y2: 1.0,
    });

    /// A linear easing curve.
    pub const LINEAR: Self = Self::Linear;

    /// An ease-in cubic easing curve.
    pub const EASE_IN: Self = Self::standard(EasingCurve::Cubic, EasingMode::In);
    /// An ease-out quadratic easing curve.
    pub const EASE_OUT: Self = Self::standard(EasingCurve::Quadratic, EasingMode::Out);
    /// An ease-in-out quadratic easing curve.
    pub const EASE_IN_OUT: Self = Self::standard(EasingCurve::Quadratic, EasingMode::InOut);

    /// An ease-in cubic easing curve.
    pub const EASE_IN_CUBIC: Self = Self::standard(EasingCurve::Cubic, EasingMode::In);
    /// An ease-out cubic easing curve.
    pub const EASE_OUT_CUBIC: Self = Self::standard(EasingCurve::Cubic, EasingMode::Out);
    /// An ease-in-out cubic easing curve.
    pub const EASE_IN_OUT_CUBIC: Self = Self::standard(EasingCurve::Cubic, EasingMode::InOut);

    /// Returns a standard easing curve with the given curve and mode.
    #[must_use]
    pub const fn standard(curve: EasingCurve, mode: EasingMode) -> Self {
        Self::Standard { curve, mode }
    }

    /// Samples this easing curve at normalized progress.
    #[must_use]
    pub fn sample(self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);

        match self {
            Self::Linear => progress,
            Self::Standard { curve, mode } => mode.sample(curve, progress),
            Self::CubicBezier(c) => c.sample(progress),
        }
    }
}

fn cubic_axis(parameter: f32, first: f32, second: f32) -> f32 {
    let inverse = 1.0 - parameter;

    3.0 * inverse.powi(2) * parameter * first
        + 3.0 * inverse * parameter.powi(2) * second
        + parameter.powi(3)
}

#[cfg(test)]
mod tests {
    use super::{CubicBezier, Easing, EasingCurve, EasingMode};

    #[test]
    fn sample_clamps_progress_below_zero() {
        let easing = Easing::standard(EasingCurve::Cubic, EasingMode::In);

        assert_eq!(easing.sample(-0.25), 0.0);
    }

    #[test]
    fn sample_clamps_progress_above_one() {
        let easing = Easing::standard(EasingCurve::Circ, EasingMode::Out);

        assert_eq!(easing.sample(1.25), 1.0);
    }

    #[test]
    fn cubic_easing_curves_preserve_endpoints() {
        for easing in [
            Easing::EASE_IN_CUBIC,
            Easing::EASE_OUT_CUBIC,
            Easing::EASE_IN_OUT_CUBIC,
        ] {
            assert_eq!(easing.sample(0.0), 0.0);
            assert_eq!(easing.sample(1.0), 1.0);
        }
    }

    #[test]
    fn cubic_bezier_preserves_endpoints() {
        let easing = Easing::CubicBezier(CubicBezier {
            x1: 0.2,
            y1: 0.8,
            x2: 0.8,
            y2: 0.2,
        });

        assert_eq!(easing.sample(0.0), 0.0);
        assert_eq!(easing.sample(1.0), 1.0);
    }

    #[test]
    fn cubic_bezier_linear_equivalent_curve_samples_midpoint() {
        let easing = Easing::CubicBezier(CubicBezier {
            x1: 0.0,
            y1: 0.0,
            x2: 1.0,
            y2: 1.0,
        });

        assert!((easing.sample(0.5) - 0.5).abs() <= 0.0001);
    }

    #[test]
    fn material_style_presets_stay_finite() {
        for easing in [
            Easing::MATERIAL_STANDARD,
            Easing::MATERIAL_STANDARD_ACCELERATE,
            Easing::MATERIAL_STANDARD_DECELERATE,
        ] {
            assert!(easing.sample(0.25).is_finite());
            assert!(easing.sample(0.5).is_finite());
            assert!(easing.sample(0.75).is_finite());
        }
    }

    #[test]
    fn sine_circ_and_expo_curves_preserve_endpoints() {
        for curve in [EasingCurve::Sine, EasingCurve::Circ, EasingCurve::Expo] {
            for mode in [EasingMode::In, EasingMode::Out, EasingMode::InOut] {
                let easing = Easing::standard(curve, mode);

                assert_eq!(easing.sample(0.0), 0.0);
                assert_eq!(easing.sample(1.0), 1.0);
            }
        }
    }

    #[test]
    fn all_builtin_curves_produce_finite_output() {
        let standard_curves = [
            EasingCurve::Quadratic,
            EasingCurve::Cubic,
            EasingCurve::Sine,
            EasingCurve::Circ,
            EasingCurve::Expo,
        ];
        let modes = [EasingMode::In, EasingMode::Out, EasingMode::InOut];
        let presets = [
            Easing::LINEAR,
            Easing::MATERIAL_STANDARD,
            Easing::MATERIAL_STANDARD_ACCELERATE,
            Easing::MATERIAL_STANDARD_DECELERATE,
        ];

        for easing in presets {
            assert!(easing.sample(0.375).is_finite());
        }

        for curve in standard_curves {
            for mode in modes {
                let easing = Easing::standard(curve, mode);

                assert!(easing.sample(0.375).is_finite());
            }
        }
    }
}
