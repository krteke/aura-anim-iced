//! Type interpolation traits and implementations.

/// Interpolates a value between two endpoints.
pub trait Interpolate: Sized {
    /// Returns the value between `from` and `to` at normalized `progress`.
    fn interpolate(from: Self, to: Self, progress: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(from: Self, to: Self, progress: f32) -> Self {
        let progress = clamp_progress(progress);

        from + (to - from) * progress
    }
}

impl Interpolate for f64 {
    fn interpolate(from: Self, to: Self, progress: f32) -> Self {
        let progress = f64::from(clamp_progress(progress));

        from + (to - from) * progress
    }
}

fn clamp_progress(progress: f32) -> f32 {
    progress.clamp(0.0, 1.0)
}
