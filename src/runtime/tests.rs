use float_cmp::assert_approx_eq;

use super::{
    AnimationClock, AnimationPlaybackState, AnimationRuntime, AnimationTargetId, AnimationTick,
    TestClock, TickPolicy,
};
use crate::{
    keyframes::{Keyframes, KeyframesBuilder},
    property::{OPACITY, PropertySnapshot, PropertyValue, SCALE, WIDTH},
    timeline::{Timeline, Track},
    timing::{Direction, Duration, Timing},
};

fn keyframes(
    spec: crate::property::PropertySpec<crate::property::Scalar>,
    from: f32,
    to: f32,
) -> Keyframes {
    KeyframesBuilder::new()
        .with_timing(Timing::new(100.0))
        .at(0.0, (spec, from))
        .at(1.0, (spec, to))
        .finish()
}

fn scalar(
    snapshot: &PropertySnapshot,
    spec: crate::property::PropertySpec<crate::property::Scalar>,
) -> f32 {
    let Some(entry) = snapshot.find_property(&spec.raw()) else {
        panic!("expected scalar property {}", spec.raw().key().name());
    };
    let PropertyValue::Scalar(value) = entry.value() else {
        panic!("expected scalar value");
    };

    *value
}

#[test]
fn runtime_stores_clock_policy_and_reports_idle_state() {
    let clock = TestClock::at(Duration::from_millis(250.0));
    let mut runtime = AnimationRuntime::with_clock(clock);

    assert!(runtime.is_idle());
    assert_eq!(runtime.active_count(), 0);
    assert_eq!(runtime.clock().now(), Duration::from_millis(250.0));
    assert!(!runtime.should_subscribe());

    let policy = TickPolicy::new(Duration::from_millis(33.0));
    runtime.set_motion_policy(policy);

    assert_eq!(runtime.motion_policy(), policy);
}

#[test]
fn registration_returns_handle_initial_snapshot_and_completion_for_zero_duration() {
    let mut runtime = AnimationRuntime::with_clock(TestClock::at(Duration::from_millis(10.0)));
    let target = AnimationTargetId::new();

    let registration = runtime.register_keyframes(
        target,
        KeyframesBuilder::new()
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .finish(),
    );

    assert!(registration.handle().id() > 0);
    assert_eq!(registration.state(), AnimationPlaybackState::Completed);
    assert_eq!(
        registration.completed_at(),
        Some(Duration::from_millis(10.0))
    );
    assert_approx_eq!(
        f32,
        scalar(
            registration.properties().expect("initial properties"),
            OPACITY
        ),
        1.0,
        epsilon = 1e-5
    );
    assert_eq!(runtime.active_count(), 0);
    assert!(runtime.is_idle());
}

#[test]
fn tick_keeps_property_composition_scoped_to_each_target() {
    let mut runtime = AnimationRuntime::testing();
    let first = AnimationTargetId::new();
    let second = AnimationTargetId::new();

    runtime.register_keyframes(first, keyframes(OPACITY, 0.0, 1.0));
    runtime.register_keyframes(first, keyframes(SCALE, 1.0, 2.0));
    runtime.register_keyframes(second, keyframes(OPACITY, 10.0, 20.0));

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let tick = runtime.tick();

    let first_snapshot = tick.properties_for(first).expect("first target");
    let second_snapshot = tick.properties_for(second).expect("second target");

    assert_eq!(tick.properties().targets().count(), 2);
    assert_approx_eq!(f32, scalar(first_snapshot, OPACITY), 0.5, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(first_snapshot, SCALE), 1.5, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(second_snapshot, OPACITY), 15.0, epsilon = 1e-5);
    assert_eq!(second_snapshot.find_property(&SCALE.raw()), None);
}

#[test]
fn tick_into_reuses_output_and_matches_tick_shape() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut output = AnimationTick::empty();

    runtime.register_keyframes(target, keyframes(OPACITY, 0.0, 1.0));

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    runtime.tick_into(&mut output);

    assert_eq!(output.timestamp(), Duration::from_millis(40.0));
    assert_approx_eq!(
        f32,
        scalar(
            output.properties_for(target).expect("target output"),
            OPACITY
        ),
        0.4,
        epsilon = 1e-5
    );
    assert!(output.completed().is_empty());

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    runtime.tick_into(&mut output);

    assert_eq!(output.completed().len(), 1);
    assert_approx_eq!(
        f32,
        scalar(
            output.properties_for(target).expect("target output"),
            OPACITY
        ),
        1.0,
        epsilon = 1e-5
    );

    runtime.tick_into(&mut output);

    assert!(output.is_empty());
    assert!(output.completed().is_empty());
}

#[test]
fn completion_removes_only_finished_entries_and_reports_completed_handles() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let short = runtime.register_keyframes(target, keyframes(OPACITY, 0.0, 1.0));
    let long = runtime.register_timeline(
        target,
        Timeline::track(
            Track::from(SCALE, 1.0)
                .to(2.0)
                .duration(Duration::from_millis(200.0)),
        ),
    );

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let tick = runtime.tick();

    assert_eq!(tick.completed(), &[short.handle()]);
    assert_eq!(runtime.active_count(), 1);
    assert!(runtime.should_subscribe());

    let snapshot = tick.properties_for(target).expect("target output");
    assert_approx_eq!(f32, scalar(snapshot, OPACITY), 1.0, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(snapshot, SCALE), 1.5, epsilon = 1e-5);

    runtime.clock_mut().set_now(Duration::from_millis(200.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[long.handle()]);
    assert_eq!(runtime.active_count(), 0);
    assert!(!runtime.should_subscribe());
}

#[test]
fn completion_output_respects_keyframe_direction() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let registration = runtime.register_keyframes(
        target,
        KeyframesBuilder::new()
            .with_timing(Timing::new(100.0).with_direction(Direction::Reverse))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .finish(),
    );

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let tick = runtime.tick();
    let snapshot = tick.properties_for(target).expect("target output");

    assert_eq!(tick.completed(), &[registration.handle()]);
    assert_approx_eq!(f32, scalar(snapshot, OPACITY), 0.0, epsilon = 1e-5);
}

#[test]
fn pause_target_freezes_position_and_disables_subscription_gate() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();

    runtime.register_keyframes(target, keyframes(OPACITY, 0.0, 1.0));
    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let before_pause = runtime.tick();
    assert_approx_eq!(
        f32,
        scalar(
            before_pause.properties_for(target).expect("target output"),
            OPACITY
        ),
        0.4,
        epsilon = 1e-5
    );

    runtime.pause_target(target);
    assert_eq!(runtime.active_count(), 1);
    assert!(!runtime.should_subscribe());

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let paused_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        scalar(
            paused_tick.properties_for(target).expect("paused output"),
            OPACITY
        ),
        0.4,
        epsilon = 1e-5
    );
    assert!(paused_tick.completed().is_empty());
}

#[test]
fn seek_target_moves_all_sources_for_target_without_touching_other_targets() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let other = AnimationTargetId::new();

    runtime.register_keyframes(target, keyframes(OPACITY, 0.0, 1.0));
    runtime.register_keyframes(target, keyframes(WIDTH, 100.0, 200.0));
    runtime.register_keyframes(other, keyframes(OPACITY, 10.0, 20.0));

    runtime.seek_target(target, Duration::from_millis(75.0));
    let tick = runtime.tick();

    let target_snapshot = tick.properties_for(target).expect("target output");
    let other_snapshot = tick.properties_for(other).expect("other output");

    assert_approx_eq!(f32, scalar(target_snapshot, OPACITY), 0.75, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(target_snapshot, WIDTH), 175.0, epsilon = 1e-5);
    assert_approx_eq!(f32, scalar(other_snapshot, OPACITY), 10.0, epsilon = 1e-5);
}

#[test]
fn handle_level_cancel_seek_and_pause_validate_target_ownership() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let wrong_target = AnimationTargetId::new();
    let registration = runtime.register_keyframes(target, keyframes(OPACITY, 0.0, 1.0));

    assert!(!runtime.seek(
        wrong_target,
        registration.handle(),
        Duration::from_millis(50.0)
    ));
    assert!(runtime.seek(target, registration.handle(), Duration::from_millis(50.0)));

    let tick = runtime.tick();
    assert_approx_eq!(
        f32,
        scalar(tick.properties_for(target).expect("target output"), OPACITY),
        0.5,
        epsilon = 1e-5
    );

    assert!(!runtime.pause(wrong_target, registration.handle()));
    assert!(runtime.pause(target, registration.handle()));
    assert!(!runtime.cancel(wrong_target, registration.handle()));
    assert!(runtime.cancel(target, registration.handle()));
    assert_eq!(runtime.active_count(), 0);
}
