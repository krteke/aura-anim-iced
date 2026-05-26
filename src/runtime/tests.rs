use super::{
    ActiveAnimation, AnimationClock, AnimationPlaybackState, AnimationRegistry, AnimationRuntime,
    AnimationSource, MotionPolicy,
};
use crate::{
    keyframes::Keyframes,
    property::{PropertyValue, UiProperty},
    timeline::Timeline,
    timing::Duration,
};

#[derive(Debug, Clone, Copy)]
struct FixedClock {
    now: Duration,
}

impl AnimationClock for FixedClock {
    fn now(&self) -> Duration {
        self.now
    }
}

#[test]
fn runtime_stores_registry_clock_and_motion_policy() {
    let clock = FixedClock {
        now: Duration::from_millis(250.0),
    };
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
