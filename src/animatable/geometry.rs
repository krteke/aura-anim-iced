//! Iced geometry interpolation support.

use iced::{Point, Rectangle, Size, Vector};

use super::{Animatable, InterpolationProgress, interpolate_with_progress};
use crate::animatable::lerp_f32_raw;

impl Animatable for Point {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            let progress = progress.value();

            Self {
                x: lerp_f32_raw(from.x, to.x, progress),
                y: lerp_f32_raw(from.y, to.y, progress),
            }
        })
    }
}

impl Animatable for Vector {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            let progress = progress.value();

            Self {
                x: lerp_f32_raw(from.x, to.x, progress),
                y: lerp_f32_raw(from.y, to.y, progress),
            }
        })
    }
}

impl Animatable for Size {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            let progress = progress.value();

            Self {
                width: lerp_f32_raw(from.width, to.width, progress),
                height: lerp_f32_raw(from.height, to.height, progress),
            }
        })
    }
}

impl Animatable for Rectangle {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            let progress = progress.value();

            Self {
                x: lerp_f32_raw(from.x, to.x, progress),
                y: lerp_f32_raw(from.y, to.y, progress),
                width: lerp_f32_raw(from.width, to.width, progress),
                height: lerp_f32_raw(from.height, to.height, progress),
            }
        })
    }
}

pub(super) fn lerp_vector_raw(from: Vector, to: Vector, progress: f32) -> Vector {
    Vector {
        x: lerp_f32_raw(from.x, to.x, progress),
        y: lerp_f32_raw(from.y, to.y, progress),
    }
}
