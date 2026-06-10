//! Integration tests for the aura-anim facade crate.

use aura_anim::prelude::*;
use float_cmp::assert_approx_eq;

#[derive(Clone, Debug, Animatable)]
struct PanelMotion {
    opacity: f32,
    scale: f32,
}

#[derive(Clone, Debug, Animatable)]
struct Offset(f32, f32);

#[derive(Clone, Debug, Animatable)]
struct Marker;

#[test]
fn prelude_exports_core_runtime_and_derive_macro() {
    let mut runtime = MotionRuntime::new();
    let panel = runtime.motion_with(
        PanelMotion {
            opacity: 0.0,
            scale: 0.8,
        },
        Timing::new(100.0),
    );

    assert!(
        panel
            .transition_to(
                PanelMotion {
                    opacity: 1.0,
                    scale: 1.0,
                },
                &mut runtime,
            )
            .is_ok()
    );
    runtime.tick(Duration::from_millis(50.0));

    let value = panel.value(&runtime).unwrap();
    assert_approx_eq!(f32, value.opacity, 0.5, epsilon = 0.000_1);
    assert_approx_eq!(f32, value.scale, 0.9, epsilon = 0.000_1);
}

#[test]
fn prelude_exports_field_animation_api() {
    let mut runtime = MotionRuntime::new();
    let panel = runtime.motion(PanelMotion {
        opacity: 0.0,
        scale: 0.8,
    });

    panel
        .play(
            fields().animate(
                field!(PanelMotion::opacity),
                tween_to(1.0, Timing::linear(100.0)),
            ),
            &mut runtime,
        )
        .unwrap();
    runtime.tick(Duration::from_millis(50.0));

    let value = panel.value(&runtime).unwrap();
    assert_approx_eq!(f32, value.opacity, 0.5, epsilon = 0.000_1);
    assert_approx_eq!(f32, value.scale, 0.8, epsilon = 0.000_1);
}

#[test]
fn prelude_exports_target_factories_and_timing_constructors() {
    let mut runtime = MotionRuntime::new();
    let value = runtime.motion(0.0_f32);

    value
        .play(tween_to(1.0, Timing::ease_out(100.0)), &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(100.0));
    assert_approx_eq!(f32, value.value(&runtime).unwrap(), 1.0);

    value
        .play(spring_to(2.0, SpringConfig::snappy()), &mut runtime)
        .unwrap();
    value.finish(&mut runtime).unwrap();
    assert_approx_eq!(f32, value.value(&runtime).unwrap(), 2.0);
}

#[test]
fn prelude_exports_composition_types() {
    let mut sequence = Sequence::new(0.0_f32)
        .then(Tween::between(0.0, 1.0, Timing::new(25.0)))
        .then(Tween::between(1.0, 2.0, Timing::new(25.0)));

    sequence.tick(Duration::from_millis(50.0));

    assert_eq!(sequence.state(), AnimationState::Completed);
    assert_approx_eq!(f32, *sequence.value(), 2.0);
}

#[test]
fn prelude_exports_motion_binding_and_transition_helpers() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum State {
        Idle,
        Active,
    }

    let binding = MotionBinding::new(State::Idle, 0.0_f32)
        .when(State::Active, 1.0)
        .fallback(|context| context.tween(Timing::new(20.0)));
    let mut runtime = MotionRuntime::new();
    let (motion, mut state) = binding.create_motion(&mut runtime);

    binding
        .set_state(&mut state, State::Active, motion, &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(20.0));

    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 1.0);
}

#[test]
fn derive_macro_supports_tuple_structs() {
    let midpoint = Offset::interpolate(&Offset(0.0, 10.0), &Offset(10.0, 30.0), 0.5);

    assert_approx_eq!(f32, midpoint.0, 5.0);
    assert_approx_eq!(f32, midpoint.1, 20.0);
}

#[test]
fn derive_macro_supports_unit_structs() {
    let _marker = Marker::interpolate(&Marker, &Marker, 0.5);
}

#[test]
fn facade_reexports_core_and_iced_crates() {
    let duration = aura_anim::core::timing::Duration::from_millis(16.0);
    let policy = aura_anim::iced::TickPolicy::interval(std::time::Duration::from_millis(16));

    assert_approx_eq!(f64, duration.as_millis(), 16.0);
    assert_eq!(
        policy,
        aura_anim::iced::TickPolicy::Interval(std::time::Duration::from_millis(16))
    );
}
