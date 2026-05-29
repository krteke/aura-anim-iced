use float_cmp::assert_approx_eq;

use super::{Hold, Parallel, Sequence, Timeline, TimelineStep, Track};
use crate::{
    keyframes::Keyframes,
    property::{OPACITY, PropertySnapshot, PropertySpec, PropertyValue, SCALE, WIDTH},
    timing::{Delay, Duration, Easing, Timing},
};

fn track(
    spec: PropertySpec<crate::property::Scalar>,
    duration_ms: f64,
    from: f32,
    to: f32,
) -> Track {
    Track::from(spec, from)
        .to(to)
        .duration(Duration::from_millis(duration_ms))
}

fn scalar(snapshot: &PropertySnapshot, spec: PropertySpec<crate::property::Scalar>) -> f32 {
    let Some(entry) = snapshot.find_property(&spec.raw()) else {
        panic!("expected scalar property {}", spec.raw().key().name());
    };
    let PropertyValue::Scalar(value) = entry.value() else {
        panic!("expected scalar value");
    };

    *value
}

#[test]
fn timeline_starts_empty_and_samples_nothing() {
    let timeline = Timeline::new().with_name("empty");

    assert_eq!(timeline.name(), Some("empty"));
    assert!(timeline.root().steps().is_empty());
    assert_eq!(timeline.total_duration(), Some(Duration::ZERO));
    assert_eq!(timeline.sample_at(Duration::ZERO), None);
    assert_eq!(timeline.completion_snapshot(), None);
}

#[test]
fn track_builder_preserves_timing_options_when_duration_is_changed() {
    let keyframes = Keyframes::new()
        .with_timing(
            Timing::new(10.0)
                .with_delay(Delay::from_millis(25.0))
                .with_easing(Easing::EaseOut),
        )
        .opacity(0.0, 0.0)
        .opacity(1.0, 1.0);

    let track = Track::new(keyframes)
        .with_name("fade")
        .duration(Duration::from_millis(100.0));

    assert_eq!(track.name(), Some("fade"));
    assert_eq!(track.total_duration(), Some(Duration::from_millis(125.0)));

    let sampled = track
        .sample_at(Duration::from_millis(75.0))
        .expect("active sample after delay");
    assert_approx_eq!(
        f32,
        scalar(&sampled, OPACITY),
        Easing::EaseOut.value(0.5),
        epsilon = 1e-5
    );
}

#[test]
fn sequence_duration_sums_steps_and_hold_keeps_previous_snapshot() {
    let sequence = Sequence::new()
        .track(track(OPACITY, 100.0, 0.0, 1.0))
        .hold(Duration::from_millis(50.0))
        .track(track(SCALE, 100.0, 1.0, 2.0));

    assert_eq!(sequence.steps().len(), 3);
    assert_eq!(
        sequence.total_duration(),
        Some(Duration::from_millis(250.0))
    );

    let held = sequence
        .sample_at(Duration::from_millis(125.0))
        .expect("hold should expose previous completion");
    assert_approx_eq!(f32, scalar(&held, OPACITY), 1.0, epsilon = 1e-5);
    assert_eq!(held.find_property(&SCALE.raw()), None);

    let second = sequence
        .sample_at(Duration::from_millis(200.0))
        .expect("second track sample");
    assert_approx_eq!(f32, scalar(&second, SCALE), 1.5, epsilon = 1e-5);
}

#[test]
fn parallel_duration_uses_longest_step_and_merges_properties() {
    let parallel = Parallel::from_steps([
        track(OPACITY, 100.0, 0.0, 1.0).into(),
        track(SCALE, 200.0, 1.0, 3.0).into(),
        Hold::new(Duration::from_millis(250.0)).into(),
    ]);

    assert_eq!(
        parallel.total_duration(),
        Some(Duration::from_millis(250.0))
    );

    let sampled = parallel
        .sample_at(Duration::from_millis(50.0))
        .expect("parallel sample");
    assert_approx_eq!(f32, scalar(&sampled, OPACITY), 0.5, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(&sampled, SCALE), 1.5, epsilon = 1e-5);
}

#[test]
fn parallel_later_steps_override_same_property_inside_group() {
    let parallel = Parallel::from_steps([
        track(OPACITY, 100.0, 0.0, 1.0).into(),
        track(OPACITY, 100.0, 10.0, 20.0).into(),
    ]);

    let sampled = parallel
        .sample_at(Duration::from_millis(50.0))
        .expect("parallel sample");

    assert_approx_eq!(f32, scalar(&sampled, OPACITY), 15.0, epsilon = 1e-5);
}

#[test]
fn timeline_builder_composes_nested_sequence_and_parallel_steps() {
    let timeline = Timeline::parallel([
        Sequence::new()
            .track(track(OPACITY, 100.0, 0.0, 1.0))
            .hold(Duration::from_millis(25.0))
            .into(),
        Parallel::new()
            .track(track(SCALE, 150.0, 1.0, 2.0))
            .track(track(WIDTH, 150.0, 100.0, 200.0))
            .into(),
    ]);

    assert_eq!(timeline.root().steps().len(), 1);
    assert_eq!(
        timeline.total_duration(),
        Some(Duration::from_millis(150.0))
    );

    let sampled = timeline
        .sample_at(Duration::from_millis(75.0))
        .expect("timeline sample");
    assert_approx_eq!(f32, scalar(&sampled, OPACITY), 0.75, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(&sampled, SCALE), 1.5, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(&sampled, WIDTH), 150.0, epsilon = 1e-5);
}

#[test]
fn completion_snapshot_merges_final_state_from_all_visible_steps() {
    let timeline = Timeline::sequence([
        TimelineStep::from(track(OPACITY, 100.0, 0.0, 1.0)),
        TimelineStep::from(track(SCALE, 100.0, 1.0, 2.0)),
    ]);

    let completed = timeline.completion_snapshot().expect("completion snapshot");

    assert_approx_eq!(f32, scalar(&completed, OPACITY), 1.0, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(&completed, SCALE), 2.0, epsilon = 1e-5);
}
