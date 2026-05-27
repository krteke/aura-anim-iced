use float_cmp::assert_approx_eq;
use iced::{Color, Shadow, Vector};
use iced::{Point, Rectangle, Size};

use super::{Animatable, InterpolationProgress};

#[test]
fn f32_interpolation_clamps_progress() {
    assert_approx_eq!(
        f32,
        f32::interpolate(10.0, 20.0, -1.0),
        10.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(f32, f32::interpolate(10.0, 20.0, 0.5), 15.0, epsilon = 1e-5);
    assert_approx_eq!(f32, f32::interpolate(10.0, 20.0, 2.0), 20.0, epsilon = 1e-5);
}

#[test]
fn f64_interpolation_clamps_progress() {
    assert_approx_eq!(
        f64,
        f64::interpolate(2.0, 6.0, f32::NAN),
        2.0,
        epsilon = 1e-10
    );
    assert_approx_eq!(f64, f64::interpolate(2.0, 6.0, 0.25), 3.0, epsilon = 1e-10);
    assert_approx_eq!(f64, f64::interpolate(2.0, 6.0, 1.5), 6.0, epsilon = 1e-10);
}

#[test]
fn i32_interpolation_rounds_to_nearest_integer() {
    assert_eq!(i32::interpolate(0, 10, 0.24), 2);
    assert_eq!(i32::interpolate(0, 10, 0.25), 3);
    assert_eq!(i32::interpolate(10, 0, 0.25), 8);
}

#[test]
fn u8_interpolation_rounds_and_uses_endpoint_clamping() {
    assert_eq!(u8::interpolate(0, 255, -0.5), 0);
    assert_eq!(u8::interpolate(0, 255, 0.5), 128);
    assert_eq!(u8::interpolate(0, 255, 2.0), 255);
}

#[test]
fn interpolation_progress_normalizes_invalid_values() {
    assert_approx_eq!(
        f32,
        InterpolationProgress::new(f32::NAN).value(),
        0.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        InterpolationProgress::new(-0.25).value(),
        0.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        InterpolationProgress::new(1.25).value(),
        1.0,
        epsilon = 1e-5
    );
}

#[test]
fn preserves_i32_from_endpoint() {
    assert_eq!(i32::interpolate(16_777_217, 20_000_000, 0.0), 16_777_217);
}

#[test]
fn preserves_i32_to_endpoint() {
    assert_eq!(i32::interpolate(16_777_217, 20_000_001, 1.0), 20_000_001);
}

#[test]
fn interpolates_large_i32_values() {
    assert_eq!(i32::interpolate(16_777_217, 16_777_219, 0.5), 16_777_218);
}

#[test]
fn interpolates_tuple_2() {
    let from = (0.0_f32, 10.0_f32);
    let to = (10.0_f32, 20.0_f32);

    assert_eq!(<(f32, f32)>::interpolate(from, to, 0.5), (5.0, 15.0));
}

#[test]
fn interpolates_tuple_3() {
    let from = (0_u8, 10_i32, 100.0_f32);
    let to = (10_u8, 20_i32, 200.0_f32);

    assert_eq!(<(u8, i32, f32)>::interpolate(from, to, 0.5), (5, 15, 150.0));
}

#[test]
fn interpolates_tuple_4() {
    let from = (0_u8, 10_i32, 100.0_f32, 1000.0_f64);
    let to = (10_u8, 20_i32, 200.0_f32, 2000.0_f64);

    assert_eq!(
        <(u8, i32, f32, f64)>::interpolate(from, to, 0.5),
        (5, 15, 150.0, 1500.0)
    );
}

#[test]
fn tuple_start_returns_from() {
    let from = (16_777_217_i32, 10_u8);
    let to = (20_000_000_i32, 20_u8);

    assert_eq!(<(i32, u8)>::interpolate(from, to, 0.0), (16_777_217, 10));
}

#[test]
fn tuple_end_returns_to() {
    let from = (16_777_217_i32, 10_u8);
    let to = (20_000_001_i32, 20_u8);

    assert_eq!(<(i32, u8)>::interpolate(from, to, 1.0), (20_000_001, 20));
}

#[test]
fn tuple_nan_returns_from() {
    let from = (10_i32, 20_u8);
    let to = (30_i32, 40_u8);

    assert_eq!(<(i32, u8)>::interpolate(from, to, f32::NAN), (10, 20));
}

#[test]
fn interpolates_nested_tuple() {
    let from = ((0.0_f32, 10.0_f32), (20_i32, 30_u8));
    let to = ((10.0_f32, 20.0_f32), (40_i32, 50_u8));

    assert_eq!(
        <((f32, f32), (i32, u8))>::interpolate(from, to, 0.5),
        ((5.0, 15.0), (30, 40))
    );
}

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
    assert_approx_eq!(f32, sampled.offset.x, 10.0, epsilon = 1e-5);
    assert_approx_eq!(f32, sampled.offset.y, 20.0, epsilon = 1e-5);
    assert_approx_eq!(f32, sampled.blur_radius, 8.0, epsilon = 1e-5);
}

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
