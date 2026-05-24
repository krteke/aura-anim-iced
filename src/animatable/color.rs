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

#[cfg(test)]
mod tests {
    use iced::Color;

    use crate::animatable::Animatable;

    #[test]
    fn color_interpolation_samples_rgba_midpoint() {
        let from = Color {
            r: 0.0,
            g: 0.25,
            b: 0.5,
            a: 0.75,
        };
        let to = Color {
            r: 1.0,
            g: 0.75,
            b: 0.25,
            a: 0.25,
        };

        let sampled = Color::interpolate(from, to, 0.5);

        assert_eq!(
            sampled,
            Color {
                r: 0.5,
                g: 0.5,
                b: 0.375,
                a: 0.5,
            }
        );
    }

    #[test]
    fn color_interpolation_clamps_progress() {
        let from = Color::from_rgba(0.1, 0.2, 0.3, 0.4);
        let to = Color::from_rgba(0.6, 0.7, 0.8, 0.9);

        assert_eq!(Color::interpolate(from, to, -1.0), from);
        assert_eq!(Color::interpolate(from, to, f32::NAN), from);
        assert_eq!(Color::interpolate(from, to, 2.0), to);
    }
}
