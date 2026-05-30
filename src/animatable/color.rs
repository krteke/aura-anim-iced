//! Iced color interpolation support.

use iced::Color;

use super::{Animatable, InterpolationProgress, interpolate_with_progress};
use crate::animatable::lerp_f32_raw;

impl Animatable for Color {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            let progress = progress.value();

            Self {
                r: lerp_f32_raw(from.r, to.r, progress),
                g: lerp_f32_raw(from.g, to.g, progress),
                b: lerp_f32_raw(from.b, to.b, progress),
                a: lerp_f32_raw(from.a, to.a, progress),
            }
        })
    }
}

pub(super) fn lerp_color_raw(from: Color, to: Color, progress: f32) -> Color {
    Color {
        r: lerp_f32_raw(from.r, to.r, progress),
        g: lerp_f32_raw(from.g, to.g, progress),
        b: lerp_f32_raw(from.b, to.b, progress),
        a: lerp_f32_raw(from.a, to.a, progress),
    }
}
