//! Iced geometry interpolation support.

use iced::{Point, Rectangle, Size, Vector};

use super::{Animatable, InterpolationProgress, interpolate_with_progress};

impl Animatable for Point {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| Self {
            x: f32::interpolate_progress(from.x, to.x, progress),
            y: f32::interpolate_progress(from.y, to.y, progress),
        })
    }
}

impl Animatable for Vector {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| Self {
            x: f32::interpolate_progress(from.x, to.x, progress),
            y: f32::interpolate_progress(from.y, to.y, progress),
        })
    }
}

impl Animatable for Size {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| Self {
            width: f32::interpolate_progress(from.width, to.width, progress),
            height: f32::interpolate_progress(from.height, to.height, progress),
        })
    }
}

impl Animatable for Rectangle {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| Self {
            x: f32::interpolate_progress(from.x, to.x, progress),
            y: f32::interpolate_progress(from.y, to.y, progress),
            width: f32::interpolate_progress(from.width, to.width, progress),
            height: f32::interpolate_progress(from.height, to.height, progress),
        })
    }
}
