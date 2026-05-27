//! Iced shadow interpolation support.

use iced::{Color, Shadow, Vector};

use super::{Animatable, InterpolationProgress, interpolate_with_progress};

impl Animatable for Shadow {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| Self {
            color: Color::interpolate_progress(from.color, to.color, progress),
            offset: Vector::interpolate_progress(from.offset, to.offset, progress),
            blur_radius: f32::interpolate_progress(from.blur_radius, to.blur_radius, progress),
        })
    }
}
