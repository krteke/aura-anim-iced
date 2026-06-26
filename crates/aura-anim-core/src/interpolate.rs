//! Interpolation progress and built-in interpolation implementations.

use crate::traits::Interpolate;

/// A normalized interpolation progress value.
///
/// Rules:
/// - `NaN` becomes `0.0`.
/// - Values below `0.0` become `0.0`.
/// - Values above `1.0` become `1.0`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct InterpolationProgress(f32);

impl InterpolationProgress {
    /// Creates a new `InterpolationProgress` with the given progress value.
    ///
    /// `NaN` values are replaced with `0.0`, and values are clamped to `[0.0, 1.0]`.
    #[must_use]
    pub fn new(progress: f32) -> Self {
        if progress.is_nan() {
            Self(0.0)
        } else {
            Self(progress.clamp(0.0, 1.0))
        }
    }

    pub(crate) fn extrapolated(progress: f32) -> Self {
        if progress.is_finite() {
            Self(progress)
        } else {
            Self(0.0)
        }
    }

    /// Returns the raw progress value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns `true` if the progress is at the start (0.0).
    #[must_use]
    pub(crate) fn is_start(self) -> bool {
        self.0 == 0.0
    }

    /// Returns `true` if the progress is at the end (1.0).
    #[must_use]
    pub(crate) fn is_end(self) -> bool {
        (self.0 - 1.0).abs() < 1e-5
    }
}

impl From<f32> for InterpolationProgress {
    fn from(progress: f32) -> Self {
        Self::new(progress)
    }
}

impl Interpolate for f32 {
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            from + (to - from) * progress.value()
        })
    }
}

impl Interpolate for f64 {
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            let progress = f64::from(progress.value());
            from + (to - from) * progress
        })
    }
}

macro_rules! impl_interpolate_integer {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl Interpolate for $ty {
                fn interpolate_progress(
                    from: &Self,
                    to: &Self,
                    progress: InterpolationProgress,
                ) -> Self {
                    interpolate_with_progress(from, to, progress, |from, to, progress| {
                        let from = f64::from(*from);
                        let to = f64::from(*to);
                        let progress = f64::from(progress.value());

                        #[allow(
                            clippy::cast_possible_truncation,
                            reason= "The interpolated value is rounded back to the integer type. Endpoints are returned before this branch.")]
                        #[allow(
                            clippy::cast_sign_loss,
                            reason= "The interpolated value is rounded back to the integer type. Endpoints are returned before this branch.")]
                        {
                            (from + (to - from) * progress).round() as Self
                        }
                    })
                }
            }
        )+
    };
}

impl_interpolate_integer!(u8, i8, u16, i16, u32, i32);

#[inline]
fn interpolate_with_progress<T: Clone>(
    from: &T,
    to: &T,
    progress: InterpolationProgress,
    interpolate_between: impl FnOnce(&T, &T, InterpolationProgress) -> T,
) -> T {
    if progress.is_start() {
        from.clone()
    } else if progress.is_end() {
        to.clone()
    } else {
        interpolate_between(from, to, progress)
    }
}

macro_rules! impl_interpolate_tuple {
    ($($name:ident : $index:tt),+) => {
        impl<$($name),+> Interpolate for ($($name,)+)
        where
            $($name: Interpolate + Clone),+
        {
            fn interpolate_progress(
                from: &Self,
                to: &Self,
                progress: InterpolationProgress,
            ) -> Self {
                interpolate_with_progress(from, to, progress, |from, to, progress| {
                    (
                        $(
                            $name::interpolate_progress(&from.$index, &to.$index, progress),
                        )+
                    )
                })
            }
        }
    };
}

impl_interpolate_tuple!(A: 0, B: 1);
impl_interpolate_tuple!(A: 0, B: 1, C: 2);
impl_interpolate_tuple!(A: 0, B: 1, C: 2, D: 3);

impl<T, const N: usize> Interpolate for [T; N]
where
    T: Interpolate + Clone,
{
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            std::array::from_fn(|i| {
                <T as Interpolate>::interpolate_progress(&from[i], &to[i], progress)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::Interpolate;

    use super::InterpolationProgress;
    use float_cmp::assert_approx_eq;

    #[test]
    fn progress_clamps_invalid_values() {
        assert_approx_eq!(f32, InterpolationProgress::new(f32::NAN).value(), 0.0);
        assert_approx_eq!(f32, InterpolationProgress::new(-0.5).value(), 0.0);
        assert_approx_eq!(f32, InterpolationProgress::new(1.5).value(), 1.0);
    }

    #[test]
    fn progress_allows_finite_extrapolation() {
        assert_approx_eq!(f32, InterpolationProgress::extrapolated(1.5).value(), 1.5);
        assert_approx_eq!(
            f32,
            InterpolationProgress::extrapolated(f32::INFINITY).value(),
            0.0
        );
    }

    #[test]
    fn interpolates_floating_point_values() {
        assert_approx_eq!(f32, f32::interpolate(&2.0, &6.0, 0.25), 3.0);
        assert_approx_eq!(f64, f64::interpolate(&2.0, &6.0, 0.25), 3.0);
    }

    #[test]
    fn interpolates_integer_values_with_rounding() {
        assert_eq!(i32::interpolate(&0, &10, 0.26), 3);
        assert_eq!(u8::interpolate(&0, &10, 0.24), 2);
    }

    #[test]
    fn interpolates_tuples_and_arrays() {
        assert_eq!(
            <(f32, i32)>::interpolate(&(0.0, 0), &(10.0, 10), 0.5),
            (5.0, 5)
        );
        let array = <[f32; 2]>::interpolate(&[0.0, 10.0], &[10.0, 20.0], 0.5);

        assert_approx_eq!(f32, array[0], 5.0);
        assert_approx_eq!(f32, array[1], 15.0);
    }
}
