//! Public `Animatable` behavior coverage.

use aura_anim_iced::animatable::{Animatable, InterpolationProgress};

#[test]
fn scalar_rounding_samples_integer_values() {
    assert_eq!(i32::interpolate(0, 10, 0.24), 2);
    assert_eq!(i32::interpolate(0, 10, 0.25), 3);
    assert_eq!(u8::interpolate(0, 255, 0.5), 128);
}

#[test]
fn progress_values_are_clamped() {
    assert_eq!(InterpolationProgress::new(f32::NAN).value(), 0.0);
    assert_eq!(InterpolationProgress::new(-1.0).value(), 0.0);
    assert_eq!(InterpolationProgress::new(2.0).value(), 1.0);
    assert_eq!(f32::interpolate(10.0, 20.0, -1.0), 10.0);
    assert_eq!(f32::interpolate(10.0, 20.0, 2.0), 20.0);
}

#[cfg(feature = "iced")]
mod iced_values {
    use aura_anim_iced::animatable::Animatable;
    use iced::{Color, Point, Rectangle, Shadow, Size, Vector};

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
    fn color_midpoint_samples_rgba_channels() {
        let sampled = Color::interpolate(
            Color::from_rgba(0.0, 0.25, 0.5, 0.75),
            Color::from_rgba(1.0, 0.75, 0.25, 0.25),
            0.5,
        );

        assert_color_close(sampled, Color::from_rgba(0.5, 0.5, 0.375, 0.5));
    }

    #[test]
    fn geometry_midpoint_samples_iced_shapes() {
        assert_eq!(
            Point::interpolate(Point::new(0.0, 10.0), Point::new(20.0, 30.0), 0.5),
            Point::new(10.0, 20.0)
        );
        assert_eq!(
            Vector::interpolate(Vector::new(-10.0, 10.0), Vector::new(10.0, 30.0), 0.5),
            Vector::new(0.0, 20.0)
        );
        assert_eq!(
            Size::interpolate(Size::new(100.0, 200.0), Size::new(300.0, 600.0), 0.5),
            Size::new(200.0, 400.0)
        );
        assert_eq!(
            Rectangle::interpolate(
                Rectangle {
                    x: 0.0,
                    y: 10.0,
                    width: 100.0,
                    height: 200.0,
                },
                Rectangle {
                    x: 20.0,
                    y: 30.0,
                    width: 300.0,
                    height: 600.0,
                },
                0.5,
            ),
            Rectangle {
                x: 10.0,
                y: 20.0,
                width: 200.0,
                height: 400.0,
            }
        );
    }

    #[test]
    fn shadow_midpoint_samples_color_offset_and_blur() {
        let sampled = Shadow::interpolate(
            Shadow {
                color: Color::from_rgba(0.0, 0.2, 0.4, 0.6),
                offset: Vector::new(0.0, 10.0),
                blur_radius: 4.0,
            },
            Shadow {
                color: Color::from_rgba(1.0, 0.6, 0.8, 1.0),
                offset: Vector::new(20.0, 30.0),
                blur_radius: 12.0,
            },
            0.5,
        );

        assert_color_close(sampled.color, Color::from_rgba(0.5, 0.4, 0.6, 0.8));
        assert_eq!(sampled.offset, Vector::new(10.0, 20.0));
        assert_eq!(sampled.blur_radius, 8.0);
    }
}
