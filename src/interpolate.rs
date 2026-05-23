//! Type interpolation traits and implementations.

/// Interpolates a value between two endpoints.
///
/// Integer implementations round interpolated values to the nearest integer.
pub trait Interpolate: Sized {
    /// Returns the value between `from` and `to` at normalized `progress`.
    fn interpolate(from: Self, to: Self, progress: f32) -> Self {
        Self::interpolate_progress(from, to, InterpolationProgress(progress))
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
