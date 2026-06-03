use float_cmp::assert_approx_eq;
use iced::{Color, Shadow, Vector};
use iced::{Point, Rectangle, Size};

use super::{Animatable, InterpolationProgress};
use crate::color::{AnimColor, Srgba};

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
fn anim_color_srgba_interpolation_samples_midpoint() {
    let from = AnimColor::Srgba(Srgba::new(0.0, 0.25, 0.5, 0.75));
    let to = AnimColor::Srgba(Srgba::new(1.0, 0.75, 0.25, 0.25));

    assert_eq!(
        AnimColor::interpolate(from, to, 0.5),
        AnimColor::Srgba(Srgba::new(0.5, 0.5, 0.375, 0.5))
    );
}

#[test]
fn anim_color_srgba_interpolation_samples_alpha_midpoint() {
    let from = AnimColor::srgba(0.2, 0.4, 0.6, 0.0);
    let to = AnimColor::srgba(0.8, 0.6, 0.4, 1.0);

    let sampled = AnimColor::interpolate(from, to, 0.5).into_iced();

    assert_color_close(sampled, Color::from_rgba(0.5, 0.5, 0.5, 0.5));
}

#[test]
fn anim_color_srgba_interpolation_samples_dark_to_light_midpoint() {
    let from = AnimColor::srgba(0.02, 0.04, 0.06, 1.0);
    let to = AnimColor::srgba(0.92, 0.96, 1.0, 1.0);

    let sampled = AnimColor::interpolate(from, to, 0.5).into_iced();

    assert_color_close(sampled, Color::from_rgba(0.47, 0.5, 0.53, 1.0));
}

#[test]
fn anim_color_interpolation_clamps_progress() {
    let from = AnimColor::from(Color::from_rgba(0.1, 0.2, 0.3, 0.4));
    let to = AnimColor::from(Color::from_rgba(0.6, 0.7, 0.8, 0.9));

    assert_eq!(AnimColor::interpolate(from, to, -1.0), from);
    assert_eq!(AnimColor::interpolate(from, to, f32::NAN), from);
    assert_eq!(AnimColor::interpolate(from, to, 2.0), to);
}

#[test]
fn iced_color_converts_to_and_from_anim_color() {
    let iced = Color::from_rgba(0.1, 0.2, 0.3, 0.4);
    let anim = AnimColor::from(iced);

    assert_eq!(anim, AnimColor::Srgba(Srgba::new(0.1, 0.2, 0.3, 0.4)));
    assert_eq!(Color::from(anim), iced);
}

#[cfg(feature = "palette")]
#[test]
fn anim_color_oklaba_interpolation_samples_perceptual_midpoint() {
    use crate::color::tag;

    let from = Color::from_rgba(0.95, 0.12, 0.08, 0.2);
    let to = Color::from_rgba(0.05, 0.28, 0.96, 0.8);
    let from_oklaba = AnimColor::from_color::<tag::Oklaba>(from);
    let to_oklaba = AnimColor::from_color::<tag::Oklaba>(to);

    let srgb = AnimColor::interpolate(from.into(), to.into(), 0.5).into_iced();
    let oklab = AnimColor::interpolate(from_oklaba, to_oklaba, 0.5).into_iced();

    assert_color_close(srgb, Color::from_rgba(0.5, 0.2, 0.52, 0.5));
    assert_approx_eq!(f32, oklab.a, 0.5, epsilon = 1e-5);
    assert!((oklab.r - srgb.r).abs() > 0.02);
    assert!((oklab.g - srgb.g).abs() > 0.02);
    assert!((oklab.b - srgb.b).abs() > 0.02);
}

#[cfg(feature = "palette")]
#[test]
fn anim_color_oklaba_interpolation_preserves_endpoints_and_clamps_output() {
    use crate::color::tag;

    let from = Color::from_rgba(1.0, 0.0, 0.0, 0.25);
    let to = Color::from_rgba(0.0, 0.0, 1.0, 0.75);
    let from = AnimColor::from_color::<tag::Oklaba>(from);
    let to = AnimColor::from_color::<tag::Oklaba>(to);

    assert_eq!(AnimColor::interpolate(from, to, -1.0), from);
    assert_eq!(AnimColor::interpolate(from, to, f32::NAN), from);
    assert_eq!(AnimColor::interpolate(from, to, 2.0), to);

    let sampled = AnimColor::interpolate(from, to, 0.5).into_iced();
    assert!((0.0..=1.0).contains(&sampled.r));
    assert!((0.0..=1.0).contains(&sampled.g));
    assert!((0.0..=1.0).contains(&sampled.b));
    assert_approx_eq!(f32, sampled.a, 0.5, epsilon = 1e-5);
}

#[cfg(feature = "palette")]
#[test]
fn anim_color_oklaba_interpolation_handles_hue_sensitive_transition() {
    use crate::color::tag;

    let from = Color::from_rgba(1.0, 0.05, 0.0, 0.35);
    let to = Color::from_rgba(0.0, 0.15, 1.0, 0.85);
    let srgb = AnimColor::interpolate(from.into(), to.into(), 0.5).into_iced();
    let oklab = AnimColor::interpolate(
        AnimColor::from_color::<tag::Oklaba>(from),
        AnimColor::from_color::<tag::Oklaba>(to),
        0.5,
    )
    .into_iced();

    assert_color_close(srgb, Color::from_rgba(0.5, 0.1, 0.5, 0.6));
    assert_approx_eq!(f32, oklab.a, 0.6, epsilon = 1e-5);
    assert!((oklab.r - srgb.r).abs() > 0.05);
    assert!((oklab.g - srgb.g).abs() > 0.05);
    assert!((oklab.b - srgb.b).abs() > 0.05);
}

#[cfg(feature = "palette")]
#[test]
fn anim_color_oklaba_interpolation_samples_dark_to_light_transition() {
    use crate::color::tag;

    let from = AnimColor::from_color::<tag::Oklaba>(Color::from_rgba(0.01, 0.02, 0.04, 0.1));
    let to = AnimColor::from_color::<tag::Oklaba>(Color::from_rgba(0.96, 0.98, 1.0, 0.9));

    let sampled = AnimColor::interpolate(from, to, 0.5).into_iced();

    assert_approx_eq!(f32, sampled.a, 0.5, epsilon = 1e-5);
    assert!(sampled.r > 0.25 && sampled.r < 0.85);
    assert!(sampled.g > 0.25 && sampled.g < 0.85);
    assert!(sampled.b > 0.25 && sampled.b < 0.85);
}
