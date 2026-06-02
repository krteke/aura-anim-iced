//! Product-level default configuration coverage.

use aura_anim_iced::{
    defaults::{ColorInterpolationMode, DefaultMotions, SpringMotionDefaults},
    property::OPACITY,
    timing::{Duration, Easing, FillMode, TimingSampleState},
};
use float_cmp::assert_approx_eq;

#[test]
fn product_motion_defaults_produce_view_ready_timing() {
    let defaults = DefaultMotions::default();
    let timing = defaults.timing();

    assert_eq!(defaults.duration(), Duration::from_millis(180.0));
    assert_eq!(defaults.easing(), Easing::EaseOut);
    assert_eq!(defaults.fill_mode(), FillMode::Forwards);
    assert_eq!(defaults.color_interpolation(), ColorInterpolationMode::Srgb);
    assert_eq!(timing.duration(), Duration::from_millis(180.0));
    assert_eq!(timing.easing(), Easing::EaseOut);
    assert_eq!(timing.fill_mode(), FillMode::Forwards);
    assert_eq!(
        timing.normalize_elapsed(240.0).sample_state,
        TimingSampleState::ForwardsFill
    );
}

#[test]
fn product_motion_defaults_can_be_overridden_for_application_style() {
    let spring = SpringMotionDefaults::new(Duration::from_millis(360.0), 0.7, 0.002);
    let defaults = DefaultMotions::default()
        .with_duration(Duration::from_millis(240.0))
        .with_easing(Easing::EaseInOut)
        .with_fill_mode(FillMode::Both)
        .with_color_interpolation(ColorInterpolationMode::Srgb)
        .with_spring(spring);
    let timing = defaults.timing();

    assert_eq!(timing.duration(), Duration::from_millis(240.0));
    assert_eq!(timing.easing(), Easing::EaseInOut);
    assert_eq!(timing.fill_mode(), FillMode::Both);
    assert_eq!(defaults.spring(), spring);
    assert_eq!(defaults.spring().response(), Duration::from_millis(360.0));
    assert_approx_eq!(f32, defaults.spring().damping_ratio(), 0.7, epsilon = 1e-5);
    assert_approx_eq!(
        f32,
        defaults.spring().settle_epsilon(),
        0.002,
        epsilon = 1e-5
    );
}

#[test]
fn product_motion_defaults_build_behavior_rules() {
    let defaults = DefaultMotions::default().with_duration(Duration::from_millis(90.0));
    let rule = defaults.behavior(OPACITY);

    assert_eq!(rule.property(), OPACITY);
    assert_eq!(rule.timing().duration(), Duration::from_millis(90.0));
    assert_eq!(rule.timing().easing(), defaults.easing());
    assert_eq!(rule.timing().fill_mode(), defaults.fill_mode());
}
