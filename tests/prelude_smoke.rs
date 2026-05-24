use aura_anim_iced::prelude::*;

#[test]
fn public_prelude_constructs_core_v01_types() {
    struct LocalValue(f32);

    impl Animatable for LocalValue {
        fn interpolate(&self, target: &Self, progress: f32) -> Self {
            Self(self.0 + (target.0 - self.0) * progress)
        }
    }

    let property = UiProperty::Opacity;
    let value = PropertyValue::Scalar(1.0);
    let timing = Timing::new(120.0);
    let keyframes = Keyframes::<PropertyValue>::new();
    let timeline = Timeline::new();
    let runtime = AnimationRuntime::new();
    let sampled = LocalValue(0.0).interpolate(&LocalValue(10.0), 0.5);

    assert_eq!(property, UiProperty::Opacity);
    assert_eq!(value, PropertyValue::Scalar(1.0));
    assert_eq!(timing.duration_ms, 120.0);
    assert!(keyframes.frames.is_empty());
    assert_eq!(timeline.name, None);
    assert!(runtime.is_idle());
    assert_eq!(sampled.0, 5.0);
}
