//! Iced color interpolation support.

use iced::Color;

use super::{Animatable, InterpolationProgress, interpolate_with_progress};

impl Animatable for Color {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            let progress = progress.value();

            Self {
                r: f32::interpolate(from.r, to.r, progress),
                g: f32::interpolate(from.g, to.g, progress),
                b: f32::interpolate(from.b, to.b, progress),
                a: f32::interpolate(from.a, to.a, progress),
            }
        })
    }
}
