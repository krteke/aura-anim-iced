//! Type interpolation traits and implementations.

/// Interpolates a value between two endpoints.
///
/// Integer implementations round interpolated values to the nearest integer.
///
/// # Examples
///
/// Implement interpolation for a custom UI value:
///
/// ```
/// use aura_anim_iced::interpolate::Interpolate;
///
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// struct Offset {
///     x: f32,
///     y: f32,
/// }
///
/// impl Interpolate for Offset {
///     fn interpolate_progress(
///         from: Self,
///         to: Self,
///         progress: aura_anim_iced::interpolate::InterpolationProgress,
///     ) -> Self {
///         Self {
///             x: f32::interpolate_progress(from.x, to.x, progress),
///             y: f32::interpolate_progress(from.y, to.y, progress),
///         }
///     }
/// }
///
/// let value = Offset::interpolate(
///     Offset { x: 0.0, y: 10.0 },
///     Offset { x: 10.0, y: 20.0 },
///     0.5,
/// );
///
/// assert_eq!(value, Offset { x: 5.0, y: 15.0 });
/// ```
pub trait Interpolate: Sized {
    /// Returns the value between `from` and `to` at normalized `progress`.
    fn interpolate(from: Self, to: Self, progress: f32) -> Self {
        Self::interpolate_progress(from, to, InterpolationProgress::new(progress))
    }

    /// Interpolates with a pre-normalized progress value.
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self;
}

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

    /// Returns the raw progress value.
    #[must_use]
    pub fn value(self) -> f32 {
        self.0
    }

    /// Returns `true` if the progress is at the start (0.0).
    #[must_use]
    pub fn is_start(self) -> bool {
        self.0 == 0.0
    }

    /// Returns `true` if the progress is at the end (1.0).
    #[must_use]
    pub fn is_end(self) -> bool {
        self.0 >= 1.0
    }
}

impl From<f32> for InterpolationProgress {
    fn from(progress: f32) -> Self {
        Self::new(progress)
    }
}

impl Interpolate for f32 {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            from + (to - from) * progress.value()
        })
    }
}

impl Interpolate for f64 {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
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
                    from: Self,
                    to: Self,
                    progress: InterpolationProgress,
                ) -> Self {
                    interpolate_with_progress(from, to, progress, |from, to, progress| {
                        let from = f64::from(from);
                        let to = f64::from(to);
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

fn interpolate_with_progress<T>(
    from: T,
    to: T,
    progress: InterpolationProgress,
    interpolate_between: impl FnOnce(T, T, InterpolationProgress) -> T,
) -> T {
    if progress.is_start() {
        from
    } else if progress.is_end() {
        to
    } else {
        interpolate_between(from, to, progress)
    }
}

macro_rules! impl_interpolate_tuple {
    ($($name:ident : $index:tt),+) => {
        impl<$($name),+> Interpolate for ($($name,)+)
        where
            $($name: Interpolate),+
        {
            fn interpolate_progress(
                from: Self,
                to: Self,
                progress: InterpolationProgress,
            ) -> Self {
                interpolate_with_progress(from, to, progress, |from, to, progress| {
                    (
                        $(
                            $name::interpolate_progress(from.$index, to.$index, progress),
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

#[cfg(test)]
mod tests {
    use super::{Interpolate, InterpolationProgress};

    #[test]
    fn is_start() {
        assert!(InterpolationProgress::new(0.0).is_start());
    }

    #[test]
    fn is_end() {
        assert!(InterpolationProgress::new(1.0).is_end());
    }

    #[test]
    fn normalizes_nan_to_start() {
        assert_eq!(InterpolationProgress::new(f32::NAN).value(), 0.0);
    }

    #[test]
    fn normalizes_negative_to_start() {
        assert_eq!(InterpolationProgress::new(-1.0).value(), 0.0);
    }

    #[test]
    fn normalizes_above_one_to_end() {
        assert_eq!(InterpolationProgress::new(2.0).value(), 1.0);
    }

    #[test]
    fn interpolates_f32() {
        assert_eq!(f32::interpolate(10.0, 20.0, 0.5), 15.0);
        assert_eq!(f32::interpolate(10.0, 20.0, 0.0), 10.0);
        assert_eq!(f32::interpolate(10.0, 20.0, 1.0), 20.0);
    }

    #[test]
    fn interpolates_f64() {
        assert_eq!(f64::interpolate(10.0, 20.0, 0.5), 15.0);
    }

    #[test]
    fn interpolates_u8() {
        assert_eq!(u8::interpolate(0, 10, 0.26), 3);
        assert_eq!(u8::interpolate(10, 0, 0.26), 7);
    }

    #[test]
    fn interpolates_i8() {
        assert_eq!(i8::interpolate(-10, 10, 0.5), 0);
        assert_eq!(i8::interpolate(-10, 10, 0.75), 5);
    }

    #[test]
    fn interpolates_u16() {
        assert_eq!(u16::interpolate(100, 200, 0.5), 150);
    }

    #[test]
    fn interpolates_i16() {
        assert_eq!(i16::interpolate(-100, 100, 0.5), 0);
    }

    #[test]
    fn interpolates_u32() {
        assert_eq!(u32::interpolate(1_000, 2_000, 0.5), 1_500);
    }

    #[test]
    fn interpolates_i32() {
        assert_eq!(i32::interpolate(-10, 10, 0.5), 0);
        assert_eq!(i32::interpolate(0, 10, 0.26), 3);
    }

    #[test]
    fn preserves_i32_from_endpoint() {
        assert_eq!(i32::interpolate(16_777_217, 20_000_000, 0.0), 16_777_217);
    }

    #[test]
    fn preserves_i32_to_endpoint() {
        assert_eq!(i32::interpolate(16_777_217, 20_000_001, 1.0), 20_000_001);
    }

    #[test]
    fn interpolates_large_i32_values() {
        assert_eq!(i32::interpolate(16_777_217, 16_777_219, 0.5), 16_777_218);
    }

    #[test]
    fn interpolates_tuple_2() {
        let from = (0.0_f32, 10.0_f32);
        let to = (10.0_f32, 20.0_f32);

        assert_eq!(<(f32, f32)>::interpolate(from, to, 0.5), (5.0, 15.0));
    }

    #[test]
    fn interpolates_tuple_3() {
        let from = (0_u8, 10_i32, 100.0_f32);
        let to = (10_u8, 20_i32, 200.0_f32);

        assert_eq!(<(u8, i32, f32)>::interpolate(from, to, 0.5), (5, 15, 150.0));
    }

    #[test]
    fn interpolates_tuple_4() {
        let from = (0_u8, 10_i32, 100.0_f32, 1000.0_f64);
        let to = (10_u8, 20_i32, 200.0_f32, 2000.0_f64);

        assert_eq!(
            <(u8, i32, f32, f64)>::interpolate(from, to, 0.5),
            (5, 15, 150.0, 1500.0)
        );
    }

    #[test]
    fn tuple_start_returns_from() {
        let from = (16_777_217_i32, 10_u8);
        let to = (20_000_000_i32, 20_u8);

        assert_eq!(<(i32, u8)>::interpolate(from, to, 0.0), (16_777_217, 10));
    }

    #[test]
    fn tuple_end_returns_to() {
        let from = (16_777_217_i32, 10_u8);
        let to = (20_000_001_i32, 20_u8);

        assert_eq!(<(i32, u8)>::interpolate(from, to, 1.0), (20_000_001, 20));
    }

    #[test]
    fn tuple_nan_returns_from() {
        let from = (10_i32, 20_u8);
        let to = (30_i32, 40_u8);

        assert_eq!(<(i32, u8)>::interpolate(from, to, f32::NAN), (10, 20));
    }

    #[test]
    fn interpolates_nested_tuple() {
        let from = ((0.0_f32, 10.0_f32), (20_i32, 30_u8));
        let to = ((10.0_f32, 20.0_f32), (40_i32, 50_u8));

        assert_eq!(
            <((f32, f32), (i32, u8))>::interpolate(from, to, 0.5),
            ((5.0, 15.0), (30, 40))
        );
    }
}
