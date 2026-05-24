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

#[cfg(test)]
mod tests {
    use iced::{Color, Shadow, Vector};

    use crate::animatable::Animatable;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= f32::EPSILON,
            "expected {actual} to be close to {expected}"
        );
    }

    fn assert_color_close(actual: Color, expected: Color) {
        assert_close(actual.r, expected.r);
        assert_close(actual.g, expected.g);
        assert_close(actual.b, expected.b);
        assert_close(actual.a, expected.a);
    }

    #[test]
    fn iced_shadow_interpolation_samples_midpoint() {
        let from = Shadow {
            color: Color::from_rgba(0.0, 0.2, 0.4, 0.6),
            offset: Vector::new(0.0, 10.0),
            blur_radius: 4.0,
        };
        let to = Shadow {
            color: Color::from_rgba(1.0, 0.6, 0.8, 1.0),
            offset: Vector::new(20.0, 30.0),
            blur_radius: 12.0,
        };

        let sampled = Shadow::interpolate(from, to, 0.5);

        assert_color_close(sampled.color, Color::from_rgba(0.5, 0.4, 0.6, 0.8));
        assert_eq!(sampled.offset, Vector::new(10.0, 20.0));
        assert_eq!(sampled.blur_radius, 8.0);
    }
}
