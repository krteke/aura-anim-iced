//! Product-level default configuration coverage.

use aura_anim_iced::{
    defaults::DefaultMotions,
    property::OPACITY,
    timing::{Duration, Easing, FillMode, TimingSampleState},
};
#[cfg(feature = "spring")]
use float_cmp::assert_approx_eq;

#[test]
fn product_motion_defaults_produce_view_ready_timing() {
    let defaults = DefaultMotions::default();
    let timing = defaults.timing();

    assert_eq!(defaults.duration(), Duration::from_millis(180.0));
    assert_eq!(defaults.easing(), Easing::EaseOut);
    assert_eq!(defaults.fill_mode(), FillMode::Forwards);
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
    let defaults = DefaultMotions::default()
        .with_duration(Duration::from_millis(240.0))
        .with_easing(Easing::EaseInOut)
        .with_fill_mode(FillMode::Both);
    let timing = defaults.timing();

    assert_eq!(timing.duration(), Duration::from_millis(240.0));
    assert_eq!(timing.easing(), Easing::EaseInOut);
    assert_eq!(timing.fill_mode(), FillMode::Both);
}

#[cfg(feature = "spring")]
#[test]
fn spring_feature_enables_spring_motion_defaults() {
    use aura_anim_iced::defaults::SpringMotionDefaults;

    let spring = SpringMotionDefaults::new(Duration::from_millis(360.0), 0.7);
    let defaults = DefaultMotions::default().with_spring(spring);

    assert_eq!(defaults.spring(), spring);
    assert_eq!(defaults.spring().response(), Duration::from_millis(360.0));
    assert_approx_eq!(f32, defaults.spring().damping_ratio(), 0.7, epsilon = 1e-5);
}

#[cfg(feature = "spring")]
#[test]
fn spring_motion_defaults_build_scalar_springs() {
    use aura_anim_iced::defaults::SpringMotionDefaults;

    let spring = SpringMotionDefaults::new(Duration::from_millis(360.0), 0.7);
    let sample = spring
        .scalar_with_velocity(0.0, 100.0, 50.0)
        .sample_at(Duration::from_millis(90.0));

    assert_eq!(spring.config().response(), Duration::from_millis(360.0));
    assert_approx_eq!(f32, spring.config().damping_ratio(), 0.7, epsilon = 1e-5);
    assert!(sample.value() > 0.0);
    assert!(sample.velocity().is_finite());
}

#[cfg(feature = "spring")]
#[test]
fn product_motion_defaults_build_scalar_springs() {
    let defaults = DefaultMotions::default();
    let spring = defaults.scalar_spring(0.0, 1.0);
    let sample = spring.sample_at(Duration::from_millis(140.0));

    assert_eq!(spring.config(), defaults.spring_config());
    assert_eq!(spring.config().response(), defaults.spring().response());
    assert_approx_eq!(
        f32,
        spring.config().damping_ratio(),
        defaults.spring().damping_ratio(),
        epsilon = 1e-5
    );
    assert!(sample.value().is_finite());
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
