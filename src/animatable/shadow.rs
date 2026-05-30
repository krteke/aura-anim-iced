//! Iced shadow interpolation support.

use iced::Shadow;

use crate::animatable::{color::lerp_color_raw, geometry::lerp_vector_raw, lerp_f32_raw};

use super::{Animatable, InterpolationProgress, interpolate_with_progress};

impl Animatable for Shadow {
    fn interpolate_progress(from: Self, to: Self, progress: InterpolationProgress) -> Self {
        interpolate_with_progress(from, to, progress, |from, to, progress| {
            let progress = progress.value();

            Self {
                color: lerp_color_raw(from.color, to.color, progress),
                offset: lerp_vector_raw(from.offset, to.offset, progress),
                blur_radius: lerp_f32_raw(from.blur_radius, to.blur_radius, progress),
            }
        })
    }
}
