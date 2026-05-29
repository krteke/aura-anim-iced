//! Compile-only smoke tests for the public prelude.

use aura_anim_iced::prelude::*;
use float_cmp::assert_approx_eq;

#[test]
fn public_prelude_constructs_core_v01_types() {
    let property = OPACITY;
    let value = PropertyValue::Scalar(1.0);
    let timing = Timing::new(120.0);
    let keyframes = Keyframes::new();
    let timeline = Timeline::new();
    let runtime = AnimationRuntime::new();

    assert_eq!(property, OPACITY);
    assert_eq!(value, PropertyValue::Scalar(1.0));
    assert_approx_eq!(f64, timing.duration().as_millis(), 120.0, epsilon = 1e-12);
    assert!(keyframes.frames().is_empty());
    assert_eq!(timeline.name(), None);
    assert!(timeline.total_duration().is_some());
    assert!(runtime.is_idle());
}
