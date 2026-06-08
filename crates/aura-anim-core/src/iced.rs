use iced_core::{Padding, Point, Rectangle, Size, Vector, border::Radius};

#[cfg(any(feature = "rgba", feature = "oklaba"))]
use iced_core::{Border, Color, Shadow};

use crate::{Interpolate, InterpolationProgress};

#[cfg(all(feature = "rgba", feature = "oklaba"))]
compile_error!("features `rgba` and `oklaba` are mutually exclusive");

macro_rules! interpolate_fields {
    ($from:ident, $to:ident, $progress:ident, $($field:ident),+ $(,)?) => {
        Self {
            $(
                $field: Interpolate::interpolate_progress(
                    &$from.$field,
                    &$to.$field,
                    $progress,
                ),
            )+
        }
    };
}

impl<T> Interpolate for Vector<T>
where
    T: Interpolate + Clone,
{
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(from, to, progress, x, y)
    }
}

impl<T> Interpolate for Point<T>
where
    T: Interpolate + Clone,
{
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(from, to, progress, x, y)
    }
}

impl<T> Interpolate for Size<T>
where
    T: Interpolate + Clone,
{
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(from, to, progress, width, height)
    }
}

impl<T> Interpolate for Rectangle<T>
where
    T: Interpolate + Clone,
{
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(from, to, progress, x, y, width, height)
    }
}

impl Interpolate for Padding {
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(from, to, progress, top, right, bottom, left)
    }
}

impl Interpolate for Radius {
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(
            from,
            to,
            progress,
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        )
    }
}

#[cfg(any(feature = "rgba", feature = "oklaba"))]
impl Interpolate for Shadow {
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(from, to, progress, color, offset, blur_radius)
    }
}

#[cfg(any(feature = "rgba", feature = "oklaba"))]
impl Interpolate for Border {
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(from, to, progress, color, width, radius)
    }
}

#[cfg(feature = "rgba")]
impl Interpolate for Color {
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        interpolate_fields!(from, to, progress, r, g, b, a)
    }
}

#[cfg(feature = "oklaba")]
impl Interpolate for Color {
    fn interpolate_progress(from: &Self, to: &Self, progress: InterpolationProgress) -> Self {
        use palette::{FromColor, Mix, Oklab, Srgb};

        let from_lab = Oklab::from_color(Srgb::new(from.r, from.g, from.b));
        let to_lab = Oklab::from_color(Srgb::new(to.r, to.g, to.b));

        let lab = from_lab.mix(to_lab, progress.value());
        let rgb = Srgb::from_color(lab);

        Color {
            r: rgb.red.clamp(0.0, 1.0),
            g: rgb.green.clamp(0.0, 1.0),
            b: rgb.blue.clamp(0.0, 1.0),
            a: f32::interpolate_progress(&from.a, &to.a, progress).clamp(0.0, 1.0),
        }
    }
}
