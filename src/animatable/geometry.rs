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

#[cfg(test)]
mod tests {
    use iced::{Point, Rectangle, Size, Vector};

    use crate::animatable::Animatable;

    #[test]
    fn point_interpolation_samples_midpoint() {
        let sampled = Point::interpolate(Point::new(0.0, 10.0), Point::new(20.0, 30.0), 0.5);

        assert_eq!(sampled, Point::new(10.0, 20.0));
    }

    #[test]
    fn vector_interpolation_samples_midpoint() {
        let sampled = Vector::interpolate(Vector::new(-10.0, 10.0), Vector::new(10.0, 30.0), 0.5);

        assert_eq!(sampled, Vector::new(0.0, 20.0));
    }

    #[test]
    fn size_interpolation_samples_midpoint() {
        let sampled = Size::interpolate(Size::new(100.0, 200.0), Size::new(300.0, 600.0), 0.5);

        assert_eq!(sampled, Size::new(200.0, 400.0));
    }

    #[test]
    fn rectangle_interpolation_samples_midpoint() {
        let from = Rectangle {
            x: 0.0,
            y: 10.0,
            width: 100.0,
            height: 200.0,
        };
        let to = Rectangle {
            x: 20.0,
            y: 30.0,
            width: 300.0,
            height: 600.0,
        };

        let sampled = Rectangle::interpolate(from, to, 0.5);

        assert_eq!(
            sampled,
            Rectangle {
                x: 10.0,
                y: 20.0,
                width: 200.0,
                height: 400.0,
            }
        );
    }

    #[test]
    fn geometry_interpolation_clamps_progress() {
        let from = Point::new(1.0, 2.0);
        let to = Point::new(3.0, 4.0);

        assert_eq!(Point::interpolate(from, to, -1.0), from);
        assert_eq!(Point::interpolate(from, to, f32::NAN), from);
        assert_eq!(Point::interpolate(from, to, 2.0), to);
    }
}
