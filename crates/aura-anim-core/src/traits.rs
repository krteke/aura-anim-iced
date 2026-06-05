use crate::interpolate::InterpolationProgress;

pub trait Interpolate: Sized {
    fn lerp(&self, other: &Self, progress: f32) -> Self {
        Self::interpolate_progress(self, other, InterpolationProgress::new(progress))
    }

    fn interpolate(from: &Self, to: &Self, progress: f32) -> Self {
        Self::interpolate_progress(from, to, InterpolationProgress::new(progress))
    }

    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self;
}

pub trait Animatable: Interpolate + Clone + 'static {}

impl<T: Interpolate + Clone + 'static> Animatable for T {}

pub trait Update {
    fn update(&mut self, dt: f64) -> bool;
}

pub trait Playable: Update {
    // milliseconds
    fn duration(&self) -> f32;

    fn seek(&mut self, progress: f32);

    fn is_complete(&self) -> bool;
}
