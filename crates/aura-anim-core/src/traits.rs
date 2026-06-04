use crate::interpolate::InterpolationProgress;

pub trait Interpolate: Sized {
    fn interpolate(from: Self, to: Self, progress: f32) -> Self {
        Self::interpolate_progress(from, to, InterpolationProgress::new(progress))
    }

    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self;
}

pub trait Animatable: Interpolate + Clone + 'static {}

impl<T: Interpolate + Clone + 'static> Animatable for T {}

pub trait Update {
    fn update(&mut self, dt: f32) -> bool;
}

pub trait Playable: Update {
    // milliseconds
    fn duration(&self) -> f32;

    fn seek(&mut self, progress: f32);

    fn is_complete(&self) -> bool;
}
