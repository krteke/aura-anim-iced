#[must_use]
pub fn clamp_progress(progress: f32) -> f32 {
    progress.clamp(0.0, 1.0)
}

pub trait Animatable: Sized {
    fn interpolate(&self, target: &Self, progress: f32) -> Self;
}
