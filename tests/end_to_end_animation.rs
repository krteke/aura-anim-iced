//! End-to-end integration coverage across keyframes, timelines, runtime ticks, and subscription gates.

use aura_anim_iced::{
    iced_ext,
    keyframes::KeyframesBuilder,
    property::{OPACITY, PropertySnapshot, PropertySpec, PropertyValue, SCALE, WIDTH},
    runtime::{AnimationRuntime, AnimationTargetId},
    timeline::{Hold, Parallel, Sequence, Timeline, Track},
    timing::{Duration, Easing, Timing},
};
use float_cmp::assert_approx_eq;

fn scalar(
    snapshot: &PropertySnapshot,
    spec: PropertySpec<aura_anim_iced::property::Scalar>,
) -> f32 {
    let Some(entry) = snapshot.find_property(&spec.raw()) else {
        panic!("expected scalar property {}", spec.raw().key().name());
    };
    let PropertyValue::Scalar(value) = entry.value() else {
        panic!("expected scalar value");
    };

    *value
}

fn scalar_track(
    spec: PropertySpec<aura_anim_iced::property::Scalar>,
    from: f32,
    to: f32,
    duration_ms: f64,
) -> Track {
    Track::new(
        KeyframesBuilder::new()
            .with_timing(Timing::new(duration_ms).with_easing(Easing::EaseInOut))
            .at(0.0, (spec, from))
            .at(1.0, (spec, to))
            .finish(),
    )
}

#[test]
fn keyframes_sequence_parallel_and_runtime_ticks_work_together() {
    let mut runtime = AnimationRuntime::testing();
    let keyframes_target = AnimationTargetId::new();
    let sequence_target = AnimationTargetId::new();
    let parallel_target = AnimationTargetId::new();

    runtime.register_keyframes(
        keyframes_target,
        KeyframesBuilder::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .finish(),
    );
    runtime.register_timeline(
        sequence_target,
        Timeline::sequence([
            scalar_track(OPACITY, 0.0, 1.0, 100.0).into(),
            Hold::new(Duration::from_millis(50.0)).into(),
            scalar_track(SCALE, 1.0, 2.0, 100.0).into(),
        ]),
    );
    runtime.register_timeline(
        parallel_target,
        Timeline::parallel([
            scalar_track(OPACITY, 10.0, 20.0, 100.0).into(),
            scalar_track(WIDTH, 100.0, 200.0, 100.0).into(),
        ]),
    );

    assert!(iced_ext::should_subscribe(&runtime));

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let first_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        scalar(
            first_tick
                .properties_for(keyframes_target)
                .expect("keyframes target"),
            OPACITY
        ),
        0.5,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        scalar(
            first_tick
                .properties_for(parallel_target)
                .expect("parallel target"),
            OPACITY
        ),
        15.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        scalar(
            first_tick
                .properties_for(parallel_target)
                .expect("parallel target"),
            WIDTH
        ),
        150.0,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(125.0));
    let hold_tick = runtime.tick();
    let sequence_hold = hold_tick
        .properties_for(sequence_target)
        .expect("sequence hold output");

    assert_approx_eq!(f32, scalar(sequence_hold, OPACITY), 1.0, epsilon = 1e-5);
    assert_eq!(sequence_hold.find_property(&SCALE.raw()), None);

    runtime.clock_mut().set_now(Duration::from_millis(250.0));
    let final_tick = runtime.tick();

    assert_eq!(runtime.active_count(), 0);
    assert!(runtime.is_idle());
    assert!(!iced_ext::should_subscribe(&runtime));
    assert!(!final_tick.completed().is_empty());
}

#[test]
fn nested_sequence_and_parallel_can_be_sampled_without_runtime() {
    let sequence = Sequence::new()
        .track(scalar_track(OPACITY, 0.0, 1.0, 100.0))
        .hold(Duration::from_millis(25.0));
    let timeline = Timeline::parallel([Parallel::new()
        .track(scalar_track(SCALE, 1.0, 2.0, 200.0))
        .sequence(sequence)
        .into()]);

    let sampled = timeline
        .sample_at(Duration::from_millis(50.0))
        .expect("nested timeline output");

    assert!(sampled.find_property(&OPACITY.raw()).is_some());
    assert!(sampled.find_property(&SCALE.raw()).is_some());
}
