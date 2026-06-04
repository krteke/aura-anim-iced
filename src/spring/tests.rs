use float_cmp::assert_approx_eq;

use super::{ScalarSpring, SpringConfig};
use crate::timing::Duration;

#[test]
fn scalar_spring_starts_at_initial_value() {
    let spring = ScalarSpring::new(
        0.0,
        100.0,
        SpringConfig::new(Duration::from_millis(320.0), 1.0),
    )
    .with_initial_velocity(12.0);
    let sample = spring.sample_at(Duration::ZERO);

    assert_approx_eq!(f32, sample.value(), 0.0, epsilon = 1e-5);
    assert_approx_eq!(f32, sample.velocity(), 12.0, epsilon = 1e-5);
}

#[test]
fn scalar_spring_converges_toward_target_without_numeric_drift() {
    let spring = ScalarSpring::new(
        0.0,
        1.0,
        SpringConfig::new(Duration::from_millis(280.0), 0.82),
    );
    let early = spring.sample_at(Duration::from_millis(70.0));
    let late = spring.sample_at(Duration::from_millis(1_400.0));

    assert!(early.value() > 0.0);
    assert!((late.value() - 1.0).abs() < 0.001);
    assert!(late.velocity().abs() < 0.02);
}

#[test]
fn scalar_spring_can_overshoot_when_underdamped() {
    let spring = ScalarSpring::new(
        0.0,
        1.0,
        SpringConfig::new(Duration::from_millis(300.0), 0.35),
    );

    let overshoot = (1..30)
        .map(|index| spring.sample_at(Duration::from_millis(f64::from(index) * 25.0)))
        .any(|sample| sample.value() > 1.0);

    assert!(overshoot);
}

#[test]
fn scalar_spring_does_not_overshoot_when_critically_damped() {
    let spring = ScalarSpring::new(
        0.0,
        1.0,
        SpringConfig::new(Duration::from_millis(300.0), 1.0),
    );

    for index in 0..30 {
        let sample = spring.sample_at(Duration::from_millis(f64::from(index) * 25.0));

        assert!(sample.value() <= 1.0);
    }
}

#[test]
fn scalar_spring_uses_initial_velocity() {
    let base = ScalarSpring::new(
        0.0,
        100.0,
        SpringConfig::new(Duration::from_millis(320.0), 0.9),
    );
    let pushed = base.with_initial_velocity(500.0);

    let base_sample = base.sample_at(Duration::from_millis(40.0));
    let pushed_sample = pushed.sample_at(Duration::from_millis(40.0));

    assert!(pushed_sample.value() > base_sample.value());
    assert!(pushed_sample.velocity() > base_sample.velocity());
}

#[test]
fn scalar_spring_zero_response_snaps_to_target() {
    let spring = ScalarSpring::new(20.0, 80.0, SpringConfig::new(Duration::ZERO, 0.6))
        .with_initial_velocity(400.0);
    let sample = spring.sample_at(Duration::from_millis(16.0));

    assert_approx_eq!(f32, sample.value(), 80.0, epsilon = 1e-5);
    assert_approx_eq!(f32, sample.velocity(), 0.0, epsilon = 1e-5);
}

#[test]
fn scalar_spring_sampling_is_deterministic() {
    let spring = ScalarSpring::new(
        -24.0,
        48.0,
        SpringConfig::new(Duration::from_millis(260.0), 0.74),
    )
    .with_initial_velocity(-60.0);

    assert_eq!(
        spring.sample_at(Duration::from_millis(123.0)),
        spring.sample_at(Duration::from_millis(123.0))
    );
}
