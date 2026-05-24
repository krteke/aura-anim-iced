//! Compile-only smoke tests for the public prelude.

use aura_anim_iced::prelude::*;

#[test]
fn public_prelude_constructs_core_v01_types() {
    struct LocalValue(f32);

    impl Animatable for LocalValue {
        fn interpolate_progress(
            from: Self,
            to: Self,
            progress: aura_anim_iced::animatable::InterpolationProgress,
        ) -> Self {
            Self(from.0 + (to.0 - from.0) * progress.value())
        }
    }

    let property = UiProperty::Opacity;
    let value = PropertyValue::Scalar(1.0);
    let timing = Timing::new(120.0);
    let keyframes = Keyframes::<PropertyValue>::new();
    let timeline = Timeline::new();
    let runtime = AnimationRuntime::new();
    let sampled = LocalValue::interpolate(LocalValue(0.0), LocalValue(10.0), 0.5);

    assert_eq!(property, UiProperty::Opacity);
    assert_eq!(value, PropertyValue::Scalar(1.0));
    assert_eq!(timing.duration_ms, 120.0);
    assert!(keyframes.frames.is_empty());
    assert_eq!(timeline.name, None);
    assert!(runtime.is_idle());
    assert_eq!(sampled.0, 5.0);
}
