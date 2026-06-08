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

#[cfg(test)]
mod tests {
    use crate::Interpolate;
    use float_cmp::assert_approx_eq;
    use iced_core::{Padding, Point, Rectangle, Size, Vector, border::Radius};

    #[test]
    fn interpolates_geometry_types() {
        let vector =
            Vector::interpolate(&Vector::new(0.0_f32, 10.0), &Vector::new(10.0, 30.0), 0.5);
        let point = Point::interpolate(&Point::new(0.0_f32, 10.0), &Point::new(10.0, 30.0), 0.5);
        let size = Size::interpolate(&Size::new(10.0_f32, 20.0), &Size::new(30.0, 60.0), 0.5);
        let rectangle = Rectangle::interpolate(
            &Rectangle::new(Point::new(0.0_f32, 10.0), Size::new(20.0, 30.0)),
            &Rectangle::new(Point::new(10.0, 30.0), Size::new(40.0, 70.0)),
            0.5,
        );

        assert_approx_eq!(f32, vector.x, 5.0);
        assert_approx_eq!(f32, vector.y, 20.0);
        assert_approx_eq!(f32, point.x, 5.0);
        assert_approx_eq!(f32, point.y, 20.0);
        assert_approx_eq!(f32, size.width, 20.0);
        assert_approx_eq!(f32, size.height, 40.0);
        assert_approx_eq!(f32, rectangle.x, 5.0);
        assert_approx_eq!(f32, rectangle.y, 20.0);
        assert_approx_eq!(f32, rectangle.width, 30.0);
        assert_approx_eq!(f32, rectangle.height, 50.0);
    }

    #[test]
    fn interpolates_padding_and_radius() {
        let padding = Padding::interpolate(
            &Padding {
                top: 0.0,
                right: 10.0,
                bottom: 20.0,
                left: 30.0,
            },
            &Padding {
                top: 10.0,
                right: 30.0,
                bottom: 50.0,
                left: 70.0,
            },
            0.5,
        );
        let radius = Radius::interpolate(
            &Radius {
                top_left: 0.0,
                top_right: 10.0,
                bottom_right: 20.0,
                bottom_left: 30.0,
            },
            &Radius {
                top_left: 10.0,
                top_right: 30.0,
                bottom_right: 50.0,
                bottom_left: 70.0,
            },
            0.5,
        );

        assert_approx_eq!(f32, padding.top, 5.0);
        assert_approx_eq!(f32, padding.right, 20.0);
        assert_approx_eq!(f32, padding.bottom, 35.0);
        assert_approx_eq!(f32, padding.left, 50.0);
        assert_approx_eq!(f32, radius.top_left, 5.0);
        assert_approx_eq!(f32, radius.top_right, 20.0);
        assert_approx_eq!(f32, radius.bottom_right, 35.0);
        assert_approx_eq!(f32, radius.bottom_left, 50.0);
    }

    #[cfg(any(feature = "rgba", feature = "oklaba"))]
    #[test]
    fn interpolates_border_and_shadow_fields() {
        use iced_core::{Border, Color, Shadow};

        let border = Border::interpolate(
            &Border {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
                width: 2.0,
                radius: Radius::from(4.0),
            },
            &Border {
                color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
                width: 6.0,
                radius: Radius::from(12.0),
            },
            0.5,
        );
        let shadow = Shadow::interpolate(
            &Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.0),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            &Shadow {
                color: Color::from_rgba(1.0, 1.0, 1.0, 1.0),
                offset: Vector::new(10.0, 6.0),
                blur_radius: 12.0,
            },
            0.5,
        );

        assert_approx_eq!(f32, border.width, 4.0);
        assert_approx_eq!(f32, border.radius.top_left, 8.0);
        assert_approx_eq!(f32, border.color.a, 0.5);
        assert_approx_eq!(f32, shadow.offset.x, 5.0);
        assert_approx_eq!(f32, shadow.offset.y, 4.0);
        assert_approx_eq!(f32, shadow.blur_radius, 8.0);
        assert_approx_eq!(f32, shadow.color.a, 0.5);
    }

    #[cfg(feature = "rgba")]
    #[test]
    fn rgba_color_interpolates_each_channel() {
        use iced_core::Color;

        let color = Color::interpolate(
            &Color::from_rgba(0.0, 0.2, 0.4, 0.6),
            &Color::from_rgba(1.0, 0.6, 0.8, 1.0),
            0.5,
        );

        assert_approx_eq!(f32, color.r, 0.5);
        assert_approx_eq!(f32, color.g, 0.4);
        assert_approx_eq!(f32, color.b, 0.6);
        assert_approx_eq!(f32, color.a, 0.8);
    }

    #[cfg(feature = "oklaba")]
    #[test]
    fn oklaba_color_interpolation_is_finite_and_preserves_alpha_progress() {
        use iced_core::Color;

        let color = Color::interpolate(
            &Color::from_rgba(1.0, 0.0, 0.0, 0.2),
            &Color::from_rgba(0.0, 0.0, 1.0, 0.8),
            0.5,
        );

        assert!(color.r.is_finite());
        assert!(color.g.is_finite());
        assert!(color.b.is_finite());
        assert!((0.0..=1.0).contains(&color.r));
        assert!((0.0..=1.0).contains(&color.g));
        assert!((0.0..=1.0).contains(&color.b));
        assert_approx_eq!(f32, color.a, 0.5);
    }
}
