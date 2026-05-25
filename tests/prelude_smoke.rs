//! Compile-only smoke tests for the public prelude.

use aura_anim_iced::prelude::*;

#[test]
fn public_prelude_constructs_core_v01_types() {
    let property = UiProperty::Opacity;
    let value = PropertyValue::Scalar(1.0);
    let timing = Timing::new(120.0);
    let keyframes = Keyframes::new();
    let timeline = Timeline::new();
    let runtime = AnimationRuntime::new();

    assert_eq!(property, UiProperty::Opacity);
    assert_eq!(value, PropertyValue::Scalar(1.0));
    assert_eq!(timing.duration().as_millis(), 120.0);
    assert!(keyframes.frames.is_empty());
    assert_eq!(timeline.name, None);
    assert!(runtime.is_idle());
}
