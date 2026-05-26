use float_cmp::assert_approx_eq;

use super::{
    ActiveAnimation, AnimationClock, AnimationPlaybackState, AnimationRegistry, AnimationRuntime,
    AnimationSource, MotionPolicy, TestClock,
};
use crate::{
    keyframes::Keyframes,
    property::{PropertySnapshot, PropertyValue, UiProperty},
    timeline::{Timeline, Track},
    timing::{Duration, Timing},
};

fn scalar(snapshot: &PropertySnapshot, target: UiProperty) -> f32 {
    let Some((_, PropertyValue::Scalar(value))) =
        snapshot.iter().find(|(property, _)| *property == target)
    else {
        panic!("expected scalar property");
    };

    *value
}

#[test]
fn runtime_stores_registry_clock_and_motion_policy() {
    let clock = TestClock::at(Duration::from_millis(250.0));
    let mut runtime = AnimationRuntime::with_clock(clock);

    assert!(runtime.is_idle());
    assert_eq!(runtime.active_count(), 0);
    assert_eq!(runtime.clock().now(), Duration::from_millis(250.0));
    assert!(!runtime.motion_policy().reduced_motion());

    let policy = MotionPolicy::new(true, Duration::from_millis(33.0));
    runtime.set_motion_policy(policy);

    assert_eq!(runtime.motion_policy(), policy);
}

#[test]
fn registry_allocates_stable_handles_and_stores_active_entries() {
    let mut registry = AnimationRegistry::new();
    let first = registry.allocate_handle();
    let second = registry.allocate_handle();

    assert_ne!(first, second);
    assert_eq!(first.id(), 1);
    assert_eq!(second.id(), 2);

    let entry = ActiveAnimation::new(first, Timeline::new(), Duration::from_millis(10.0));
    let inserted = registry.insert(entry);

    assert_eq!(inserted, first);
    assert_eq!(registry.active_count(), 1);
    assert_eq!(
        registry.get(first).map(ActiveAnimation::started_at),
        Some(Duration::from_millis(10.0))
    );
    assert!(matches!(
        registry.get(first).map(ActiveAnimation::source),
        Some(AnimationSource::Timeline(_))
    ));
    assert!(registry.get(second).is_none());
}

#[test]
fn active_entry_tracks_state_and_last_snapshot() {
    let mut registry = AnimationRegistry::new();
    let handle = registry.allocate_handle();
    let keyframes = Keyframes::new().opacity(0.0, 0.0).opacity(1.0, 1.0);
    let mut entry = ActiveAnimation::new(handle, keyframes, Duration::ZERO);

    assert_eq!(entry.handle(), handle);
    assert_eq!(entry.state(), AnimationPlaybackState::Playing);
    assert!(matches!(entry.source(), AnimationSource::Keyframes(_)));
    assert!(entry.last_snapshot().is_none());

    entry.set_state(AnimationPlaybackState::Paused);
    entry.set_last_snapshot(Some(vec![(
        UiProperty::Opacity,
        PropertyValue::Scalar(0.5),
    )]));

    assert_eq!(entry.state(), AnimationPlaybackState::Paused);
    assert_eq!(
        entry.last_snapshot(),
        Some(&vec![(UiProperty::Opacity, PropertyValue::Scalar(0.5))])
    );
}

#[test]
fn registry_removes_and_clears_entries() {
    let mut registry = AnimationRegistry::new();
    let first = registry.allocate_handle();
    let second = registry.allocate_handle();

    registry.insert(ActiveAnimation::new(first, Timeline::new(), Duration::ZERO));
    registry.insert(ActiveAnimation::new(
        second,
        Keyframes::new(),
        Duration::from_millis(5.0),
    ));

    assert_eq!(registry.active_count(), 2);
    assert_eq!(
        registry.remove(first).map(|entry| entry.handle()),
        Some(first)
    );
    assert!(registry.get(first).is_none());
    assert_eq!(registry.active_count(), 1);

    registry.clear();

    assert!(registry.is_empty());
}

#[test]
fn runtime_registers_keyframes_with_start_time_and_initial_snapshot() {
    let mut runtime = AnimationRuntime::with_clock(TestClock::at(Duration::from_millis(250.0)));
    let keyframes = Keyframes::new()
        .with_timing(Timing::new(100.0))
        .opacity(0.0, 0.25)
        .opacity(1.0, 1.0);

    let registration = runtime.register_keyframes(keyframes);
    let entry = runtime
        .registry()
        .get(registration.handle())
        .expect("registered keyframes");

    assert_eq!(registration.started_at(), Duration::from_millis(250.0));
    assert_eq!(registration.state(), AnimationPlaybackState::Playing);
    assert_eq!(entry.started_at(), Duration::from_millis(250.0));
    assert_eq!(entry.state(), AnimationPlaybackState::Playing);
    assert_eq!(entry.completed_at(), None);
    assert_eq!(
        registration.properties(),
        Some(&vec![(UiProperty::Opacity, PropertyValue::Scalar(0.25))])
    );
    assert_eq!(entry.last_snapshot(), registration.properties());
}

#[test]
fn runtime_registers_timelines_with_initial_snapshot_output() {
    let mut runtime = AnimationRuntime::with_clock(TestClock::at(Duration::from_millis(80.0)));
    let timeline = Timeline::track(
        Track::from(UiProperty::Opacity, 0.0)
            .to(1.0)
            .duration(Duration::from_millis(100.0)),
    );

    let registration = runtime.register_timeline(timeline);
    let entry = runtime
        .registry()
        .get(registration.handle())
        .expect("registered timeline");

    assert_eq!(registration.started_at(), Duration::from_millis(80.0));
    assert_eq!(registration.state(), AnimationPlaybackState::Playing);
    assert!(matches!(entry.source(), AnimationSource::Timeline(_)));
    assert_eq!(
        registration.properties(),
        Some(&vec![(UiProperty::Opacity, PropertyValue::Scalar(0.0))])
    );
}

#[test]
fn runtime_marks_zero_duration_sources_completed_at_registration() {
    let mut runtime = AnimationRuntime::with_clock(TestClock::at(Duration::from_millis(400.0)));
    let keyframes = Keyframes::new().opacity(0.0, 0.0).opacity(1.0, 1.0);

    let registration = runtime.register_keyframes(keyframes);
    let entry = runtime
        .registry()
        .get(registration.handle())
        .expect("registered completed keyframes");

    assert_eq!(registration.state(), AnimationPlaybackState::Completed);
    assert_eq!(
        registration.completed_at(),
        Some(Duration::from_millis(400.0))
    );
    assert!(entry.is_completed());
    assert_eq!(entry.completed_at(), registration.completed_at());
    assert_eq!(
        registration.properties(),
        Some(&vec![(UiProperty::Opacity, PropertyValue::Scalar(1.0))])
    );
    assert_eq!(entry.last_snapshot(), registration.properties());
}

#[test]
fn runtime_tick_advances_active_animations_and_aggregates_properties() {
    let mut runtime = AnimationRuntime::testing();
    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0),
    );
    runtime.register_timeline(Timeline::track(
        Track::from(UiProperty::Scale, 1.0)
            .to(2.0)
            .duration(Duration::from_millis(100.0)),
    ));
    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 10.0)
            .opacity(1.0, 20.0),
    );

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let tick = runtime.tick();

    assert_eq!(tick.timestamp(), Duration::from_millis(50.0));
    assert_eq!(
        tick.properties()
            .iter()
            .map(|(property, _)| *property)
            .collect::<Vec<_>>(),
        vec![UiProperty::Opacity, UiProperty::Scale]
    );
    assert_eq!(runtime.active_count(), 3);
    assert!(tick.completed().is_empty());
    assert_approx_eq!(
        f32,
        scalar(tick.properties(), UiProperty::Opacity),
        15.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        scalar(tick.properties(), UiProperty::Scale),
        1.5,
        epsilon = 1e-5
    );
}

#[test]
fn runtime_tick_emits_completion_snapshot_and_removes_completed_entries() {
    let mut runtime = AnimationRuntime::testing();
    let registration = runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0),
    );

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let tick = runtime.tick();

    assert_eq!(tick.completed(), &[registration.handle()]);
    assert_eq!(runtime.active_count(), 0);
    assert_approx_eq!(
        f32,
        scalar(tick.properties(), UiProperty::Opacity),
        1.0,
        epsilon = 1e-5
    );
}

#[test]
fn runtime_idle_detection_separates_active_entries_from_tick_gate() {
    let mut runtime = AnimationRuntime::testing();

    assert_eq!(runtime.active_count(), 0);
    assert!(runtime.is_idle());
    assert!(!runtime.should_tick());
    assert!(!runtime.should_subscribe());

    let registration = runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0),
    );

    assert_eq!(runtime.active_count(), 1);
    assert!(!runtime.is_idle());
    assert!(runtime.should_tick());
    assert!(runtime.should_subscribe());

    runtime
        .registry_mut()
        .get_mut(registration.handle())
        .expect("registered entry")
        .set_state(AnimationPlaybackState::Paused);

    assert_eq!(runtime.active_count(), 1);
    assert!(!runtime.is_idle());
    assert!(!runtime.should_tick());
    assert!(!runtime.should_subscribe());

    runtime
        .registry_mut()
        .get_mut(registration.handle())
        .expect("registered entry")
        .set_state(AnimationPlaybackState::Canceled);

    assert_eq!(runtime.active_count(), 0);
    assert!(runtime.is_idle());
    assert!(!runtime.should_tick());
    assert!(!runtime.should_subscribe());
}

#[test]
fn runtime_completed_registration_is_idle_until_cleanup_tick() {
    let mut runtime = AnimationRuntime::with_clock(TestClock::at(Duration::from_millis(10.0)));

    runtime.register_keyframes(Keyframes::new().opacity(0.0, 0.0).opacity(1.0, 1.0));

    assert_eq!(runtime.registry().active_count(), 1);
    assert_eq!(runtime.active_count(), 0);
    assert!(runtime.is_idle());
    assert!(!runtime.should_subscribe());
}

#[test]
fn test_clock_injects_and_advances_deterministic_runtime_time() {
    let mut runtime = AnimationRuntime::testing();

    assert_eq!(runtime.clock().now(), Duration::ZERO);

    runtime.clock_mut().set_now(Duration::from_millis(25.0));
    assert_eq!(runtime.clock().now(), Duration::from_millis(25.0));

    runtime.clock_mut().advance_by(Duration::from_millis(75.0));
    assert_eq!(runtime.clock().now(), Duration::from_millis(100.0));
}
