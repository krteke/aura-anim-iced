//! Integration tests for the public aura-anim-core API.

use aura_anim_core::{
    Animatable, Animation, AnimationCommand, AnimationExt, AnimationState, Hold, Interpolate,
    MotionBinding, MotionError, MotionRuntime, Parallel, Presence, RetainPolicy, Sequence, Spring,
    SpringConfig, Timeline, Tween,
    keyframes::{Keyframe, Keyframes},
    timing::{Delay, Direction, Duration, Easing, IterationCount, Timing},
};
use float_cmp::assert_approx_eq;

#[derive(Clone, Debug, Animatable)]
struct Position {
    x: f32,
    y: f32,
}

fn assert_position(value: &Position, x: f32, y: f32) {
    assert_approx_eq!(f32, value.x, x, epsilon = 0.000_1);
    assert_approx_eq!(f32, value.y, y, epsilon = 0.000_1);
}

#[derive(Clone)]
struct ProgressAnimation {
    value: f32,
    state: AnimationState,
}

impl ProgressAnimation {
    fn new() -> Self {
        Self {
            value: 0.0,
            state: AnimationState::Running,
        }
    }
}

impl Animation<f32> for ProgressAnimation {
    fn value(&self) -> &f32 {
        &self.value
    }

    fn state(&self) -> AnimationState {
        self.state
    }

    #[allow(clippy::cast_possible_truncation)]
    fn tick(&mut self, delta: Duration) {
        self.value += delta.as_secs() as f32;
    }

    fn pause(&mut self) {
        self.state = AnimationState::Paused;
    }

    fn resume(&mut self) {
        self.state = AnimationState::Running;
    }

    fn cancel(&mut self) {
        self.state = AnimationState::Canceled;
    }

    fn seek(&mut self, progress: f32) {
        self.value = progress;
    }

    fn finish(&mut self) {
        self.value = 1.0;
        self.state = AnimationState::Completed;
    }
}

#[test]
fn tween_progresses_through_delay_and_iterations() {
    let timing = Timing::new(100.0)
        .with_delay(Delay::from_millis(50.0))
        .with_iterations(2)
        .with_direction(Direction::Alternate);
    let mut tween = Tween::between(0.0_f32, 10.0, timing);

    tween.tick(Duration::from_millis(25.0));
    assert_approx_eq!(f32, *tween.value(), 0.0);
    assert_eq!(tween.state(), AnimationState::Running);

    tween.tick(Duration::from_millis(75.0));
    assert_approx_eq!(f32, *tween.value(), 5.0);

    tween.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, *tween.value(), 10.0);

    tween.tick(Duration::from_millis(100.0));
    assert!(tween.is_completed());
    assert_approx_eq!(f32, *tween.value(), 0.0);
}

#[test]
fn runtime_manages_motion_lifecycle_and_retargeting() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion_with(0.0_f32, Timing::new(100.0).with_easing(Easing::Linear));

    assert!(motion.transition_to(10.0, &mut runtime).is_ok());
    assert!(runtime.has_active());
    runtime.tick(Duration::from_millis(40.0));
    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 4.0);

    assert!(motion.transition_to(14.0, &mut runtime).is_ok());
    runtime.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 9.0);

    assert!(motion.pause(&mut runtime).is_ok());
    runtime.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 9.0);

    assert!(motion.resume(&mut runtime).is_ok());
    runtime.tick(Duration::from_millis(50.0));
    assert!(motion.is_completed(&runtime).unwrap());
    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 14.0);
}

#[test]
fn motion_binding_drives_existing_motion_from_business_state() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum State {
        Idle,
        Hovered,
        Pressed,
    }

    let binding = MotionBinding::new(State::Idle, 0.0_f32)
        .when(State::Hovered, 10.0)
        .when(State::Pressed, 20.0)
        .transition(State::Idle, State::Hovered, |context| {
            Tween::between(context.from, context.to, Timing::new(100.0)).boxed()
        })
        .fallback(|context| Tween::between(context.from, context.to, Timing::new(50.0)).boxed());
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion(-10.0_f32);
    let mut state = binding.state();

    assert!(
        binding
            .set_state(&mut state, State::Hovered, motion, &mut runtime)
            .unwrap()
    );
    runtime.tick(Duration::from_millis(40.0));
    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), -2.0);

    assert!(
        binding
            .set_state(&mut state, State::Pressed, motion, &mut runtime)
            .unwrap()
    );
    runtime.tick(Duration::from_millis(50.0));

    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 20.0);
    assert_eq!(state.current(), &State::Pressed);
}

#[test]
fn play_once_drops_settled_animation() {
    let mut runtime = MotionRuntime::new();
    let retained = runtime.motion_count();
    let transient = runtime.play_once(Tween::between(0.0_f32, 1.0, Timing::new(10.0)));

    assert_eq!(runtime.motion_count(), retained + 1);
    runtime.tick(Duration::from_millis(10.0));

    assert_eq!(
        transient.value(&runtime),
        Err(MotionError::Removed { slot: 0 })
    );
    assert_eq!(runtime.motion_count(), retained);
    assert_eq!(runtime.active_count(), 0);
}

#[test]
fn keyframes_replace_duplicate_times_and_follow_direction() {
    let mut animation = Keyframes::new(0.0_f32)
        .push(100.0, 10.0)
        .push(100.0, 20.0)
        .with_iterations(2)
        .with_direction(Direction::Alternate);

    animation.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, *animation.value(), 10.0);

    animation.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, *animation.value(), 20.0);

    animation.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, *animation.value(), 10.0);
}

#[test]
fn sequence_consumes_overflow_between_children() {
    let mut sequence = Timeline::new(0.0_f32)
        .then(Tween::between(0.0, 10.0, Timing::new(100.0)))
        .then(Hold::new(10.0, Duration::from_millis(50.0)))
        .then(Tween::between(10.0, 20.0, Timing::new(100.0)));

    sequence.tick(Duration::from_millis(175.0));

    assert_approx_eq!(f32, *sequence.value(), 12.5, epsilon = 0.000_1);
    assert_eq!(sequence.state(), AnimationState::Running);

    sequence.tick(Duration::from_millis(75.0));
    assert_eq!(sequence.state(), AnimationState::Completed);
    assert_approx_eq!(f32, *sequence.value(), 20.0);
}

#[test]
fn parallel_composes_outputs_and_finishes_after_longest_child() {
    let start = Position { x: 0.0, y: 0.0 };
    let x_branch = Tween::between(
        start.clone(),
        Position { x: 10.0, y: 0.0 },
        Timing::new(100.0),
    );
    let y_branch = Tween::between(
        start.clone(),
        Position { x: 0.0, y: 20.0 },
        Timing::new(200.0),
    );
    let mut parallel = Parallel::new(start, |outputs: &[Position]| Position {
        x: outputs[0].x,
        y: outputs[1].y,
    })
    .with(x_branch)
    .with(y_branch);

    parallel.tick(Duration::from_millis(100.0));
    assert_position(parallel.value(), 10.0, 10.0);
    assert_eq!(parallel.state(), AnimationState::Running);

    parallel.tick(Duration::from_millis(100.0));
    assert_position(parallel.value(), 10.0, 20.0);
    assert_eq!(parallel.state(), AnimationState::Completed);
}

#[test]
fn presence_mounts_until_exit_animation_settles() {
    let mut runtime = MotionRuntime::new();
    let mut presence = Presence::new(&mut runtime, 0.0_f32, 1.0, Timing::new(100.0));

    assert!(!presence.is_mounted());
    presence.show(&mut runtime).unwrap();
    assert!(presence.is_mounted());
    assert!(presence.is_visible());

    runtime.tick(Duration::from_millis(100.0));
    assert_approx_eq!(f32, *presence.value(&runtime).unwrap(), 1.0);

    presence.hide(&mut runtime).unwrap();
    assert!(presence.is_mounted());
    assert!(!presence.is_visible());

    runtime.tick(Duration::from_millis(100.0));
    presence.sync(&runtime).unwrap();
    assert!(!presence.is_mounted());
    assert_approx_eq!(f32, *presence.value(&runtime).unwrap(), 0.0);
}

#[test]
fn spring_can_seek_finish_and_retarget() {
    let mut spring = Spring::new(0.0_f32, 10.0, SpringConfig::default());

    spring.seek(0.5);
    assert_approx_eq!(f32, *spring.value(), 5.0);

    spring.retarget(20.0);
    spring.tick(Duration::from_millis(16.0));
    assert_eq!(spring.state(), AnimationState::Running);

    spring.finish();
    assert_eq!(spring.state(), AnimationState::Completed);
    assert_approx_eq!(f32, *spring.value(), 20.0);
}

#[test]
fn spring_channels_support_per_field_physics() {
    let slow = SpringConfig {
        stiffness: 40.0,
        damping: 14.0,
        ..SpringConfig::default()
    };
    let fast = SpringConfig {
        stiffness: 420.0,
        damping: 28.0,
        ..SpringConfig::default()
    };
    let mut spring = Spring::with_channels(
        (0.0_f32, 0.0_f32),
        (100.0, 100.0),
        [slow, fast],
        |outputs| (outputs[0].0, outputs[1].1),
    );

    spring.tick(Duration::from_millis(100.0));

    assert_eq!(spring.channel_count(), 2);
    assert!(spring.value().1 > spring.value().0 * 2.0);
}

#[test]
fn seek_normalizes_invalid_progress_values() {
    let mut tween = Tween::between(0.0_f32, 10.0, Timing::new(100.0));

    tween.seek(f32::NAN);
    assert_approx_eq!(f32, *tween.value(), 0.0);

    tween.seek(2.0);
    assert_eq!(tween.state(), AnimationState::Completed);
    assert_approx_eq!(f32, *tween.value(), 10.0);
}

#[test]
fn infinite_animation_reports_no_finite_duration() {
    let animation = Keyframes::new(0.0_f32)
        .push(100.0, 1.0)
        .with_iterations(IterationCount::INFINITE);

    assert_eq!(Animation::duration(&animation), None);
}

#[test]
fn interpolation_clamps_and_extrapolates_public_values() {
    assert_approx_eq!(f32, f32::interpolate(&0.0, &10.0, -1.0), 0.0);
    assert_approx_eq!(f32, f32::interpolate(&0.0, &10.0, 2.0), 10.0);
    assert_approx_eq!(f32, f32::extrapolate(&0.0, &10.0, 1.5), 15.0);

    let midpoint = Position::interpolate(
        &Position { x: 0.0, y: 10.0 },
        &Position { x: 10.0, y: 30.0 },
        0.5,
    );
    assert_position(&midpoint, 5.0, 20.0);
}

#[test]
fn timing_builders_and_duration_accessors_are_consistent() {
    let timing = Timing::new(125.0)
        .with_delay(Delay::from_secs(0.025))
        .with_easing(Easing::EaseInOut)
        .with_direction(Direction::Reverse)
        .with_iterations(3);

    assert_approx_eq!(f64, timing.duration().as_millis(), 125.0);
    assert_approx_eq!(f64, timing.delay().as_millis(), 25.0);
    assert_eq!(timing.easing(), Easing::EaseInOut);
    assert_eq!(timing.direction(), Direction::Reverse);
    assert_eq!(timing.iterations().finite_count(), Some(3));
    assert_approx_eq!(f64, timing.active_duration().unwrap().as_millis(), 375.0);
    assert_approx_eq!(f64, timing.total_duration().unwrap().as_millis(), 400.0);
}

#[test]
fn animation_rate_scales_duration_based_sources_and_ignores_springs() {
    let tween = Tween::between(0.0_f32, 1.0, Timing::new(100.0)).rate(2.0);
    let keyframes = Keyframes::new(0.0_f32)
        .push(100.0, 1.0)
        .push(200.0, 2.0)
        .rate(2.0);
    let timeline = Sequence::new(0.0_f32)
        .then(Tween::between(0.0, 1.0, Timing::new(100.0)))
        .then(Hold::new(1.0, Duration::from_millis(100.0)))
        .rate(2.0);
    let spring = Spring::new(0.0_f32, 1.0, SpringConfig::default()).rate(2.0);

    assert_eq!(tween.duration(), Some(Duration::from_millis(50.0)));
    assert_eq!(keyframes.duration(), Some(Duration::from_millis(100.0)));
    assert_eq!(timeline.duration(), Some(Duration::from_millis(100.0)));
    assert_eq!(spring.duration(), None);
}

#[test]
fn infinite_timing_has_no_total_duration() {
    let timing = Timing::new(100.0).with_iterations(IterationCount::infinite());

    assert_eq!(timing.active_duration(), None);
    assert_eq!(timing.total_duration(), None);
}

#[test]
fn duration_and_delay_convert_from_standard_duration() {
    let standard = std::time::Duration::from_millis(250);
    let duration = Duration::from(standard);
    let delay = Delay::from(standard);

    assert_approx_eq!(f64, duration.as_secs(), 0.25);
    assert_approx_eq!(f64, duration.as_millis(), 250.0);
    assert_approx_eq!(f64, delay.as_millis(), 250.0);
    assert_eq!(
        duration + Duration::from_millis(50.0),
        Duration::from_millis(300.0)
    );
}

#[test]
fn keyframe_accessors_and_custom_easing_are_public() {
    let frame = Keyframe::new(80.0, 4.0_f32).with_easing(Easing::EaseOut);

    assert_approx_eq!(f64, frame.time(), 80.0);
    assert_approx_eq!(f32, *frame.value(), 4.0);
    assert_eq!(frame.easing(), Easing::EaseOut);
}

#[test]
fn keyframes_honor_delay_seek_and_finish_direction() {
    let mut animation = Keyframes::new(0.0_f32)
        .push_eased(100.0, 10.0, Easing::Linear)
        .with_delay(Delay::from_millis(50.0))
        .with_iterations(2)
        .with_direction(Direction::Alternate);

    animation.tick(Duration::from_millis(25.0));
    assert_approx_eq!(f32, *animation.value(), 0.0);

    animation.seek(0.5);
    assert_approx_eq!(f32, *animation.value(), 7.5);

    animation.finish();
    assert_eq!(animation.state(), AnimationState::Completed);
    assert_approx_eq!(f32, *animation.value(), 0.0);
}

#[test]
fn keyframes_advance_returns_overflow_after_finite_completion() {
    let mut animation = Keyframes::new(0.0_f32).push(100.0, 10.0);

    let overflow = animation.advance(Duration::from_millis(130.0));

    assert_eq!(overflow, Duration::from_millis(30.0));
    assert_eq!(animation.state(), AnimationState::Completed);
    assert_approx_eq!(f32, *animation.value(), 10.0);
}

#[test]
fn animation_trait_defaults_are_observable() {
    let mut animation = ProgressAnimation::new();

    assert_eq!(animation.duration(), None);
    assert!(animation.is_active());
    assert!(!animation.retarget(&5.0));

    let overflow = animation.advance(Duration::from_millis(500.0));
    assert_eq!(overflow, Duration::ZERO);
    assert_approx_eq!(f32, *animation.value(), 0.5);
}

#[test]
fn sequence_with_unknown_duration_seeks_by_child_index() {
    let mut sequence = Sequence::new(0.0_f32)
        .then(ProgressAnimation::new())
        .then(ProgressAnimation::new());

    assert_eq!(sequence.duration(), None);
    sequence.seek(0.75);

    assert_eq!(sequence.state(), AnimationState::Running);
    assert_approx_eq!(f32, *sequence.value(), 0.5);
}

#[test]
fn sequence_accessors_and_lifecycle_commands_work() {
    let mut sequence = Sequence::new(0.0_f32);
    assert!(sequence.is_empty());

    sequence.push(Tween::between(0.0, 1.0, Timing::new(100.0)));
    assert_eq!(sequence.len(), 1);
    assert!(!sequence.is_empty());

    sequence.pause();
    assert_eq!(sequence.state(), AnimationState::Paused);
    sequence.resume();
    assert_eq!(sequence.state(), AnimationState::Running);
    sequence.cancel();
    assert_eq!(sequence.state(), AnimationState::Canceled);
}

#[test]
fn parallel_accessors_duration_and_finish_work() {
    let mut parallel = Parallel::new(0.0_f32, |values| values.iter().sum());
    assert!(parallel.is_empty());

    parallel.push(Tween::between(0.0, 1.0, Timing::new(50.0)));
    parallel.push(Tween::between(0.0, 2.0, Timing::new(100.0)));
    assert_eq!(parallel.len(), 2);
    assert_eq!(parallel.duration(), Some(Duration::from_millis(100.0)));

    parallel.pause();
    assert_eq!(parallel.state(), AnimationState::Paused);
    parallel.resume();
    parallel.finish();

    assert_eq!(parallel.state(), AnimationState::Completed);
    assert_approx_eq!(f32, *parallel.value(), 3.0);
}

#[test]
fn parallel_with_unknown_duration_reports_none() {
    let parallel = Parallel::new(0.0_f32, |values| values.iter().sum())
        .with(ProgressAnimation::new())
        .with(Tween::between(0.0, 1.0, Timing::new(100.0)));

    assert_eq!(parallel.duration(), None);
}

#[test]
fn hold_exposes_duration_and_ignores_ticks_while_paused() {
    let mut hold = Hold::new(7_i32, Duration::from_millis(100.0));

    assert_eq!(hold.duration(), Some(Duration::from_millis(100.0)));
    hold.pause();
    hold.tick(Duration::from_millis(100.0));
    assert_eq!(hold.state(), AnimationState::Paused);
    assert_eq!(*hold.value(), 7);

    hold.resume();
    hold.tick(Duration::from_millis(100.0));
    assert_eq!(hold.state(), AnimationState::Completed);
}

#[test]
fn runtime_supports_direct_insert_play_and_commands() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.insert(
        Tween::between(0.0_f32, 10.0, Timing::new(100.0)),
        Timing::new(25.0),
    );

    assert_eq!(runtime.state(motion), Ok(AnimationState::Running));
    assert!(runtime.command(motion, AnimationCommand::Seek(0.5)).is_ok());
    assert_approx_eq!(f32, *runtime.value(motion).unwrap(), 5.0);

    assert!(runtime.command(motion, AnimationCommand::Pause).is_ok());
    assert_eq!(runtime.state(motion), Ok(AnimationState::Paused));
    assert!(runtime.command(motion, AnimationCommand::Resume).is_ok());
    assert!(
        runtime
            .play(motion, Keyframes::new(5.0_f32).push(50.0, 15.0))
            .is_ok()
    );
    runtime.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 15.0);
}

#[test]
fn runtime_finish_cancel_and_remove_commands_update_storage() {
    let mut runtime = MotionRuntime::new();
    let finished = runtime.motion(0.0_f32);
    let canceled = runtime.motion(0.0_f32);

    assert!(finished.transition_to(1.0, &mut runtime).is_ok());
    assert!(canceled.transition_to(1.0, &mut runtime).is_ok());
    assert!(finished.finish(&mut runtime).is_ok());
    assert!(canceled.cancel(&mut runtime).is_ok());
    assert_eq!(finished.state(&runtime), Ok(AnimationState::Completed));
    assert_eq!(canceled.state(&runtime), Ok(AnimationState::Canceled));

    assert!(finished.remove(&mut runtime).is_ok());
    assert!(canceled.remove(&mut runtime).is_ok());
    assert_eq!(runtime.motion_count(), 0);
    assert!(!runtime.has_active());
}

#[test]
fn stale_motion_handles_fail_without_affecting_reused_slots() {
    let mut runtime = MotionRuntime::new();
    let stale = runtime.motion(1.0_f32);
    assert!(runtime.remove(stale).is_ok());
    assert_eq!(stale.value(&runtime), Err(MotionError::Removed { slot: 0 }));

    let current = runtime.motion(2.0_f32);

    let expected_error = MotionError::StaleHandle {
        slot: 0,
        handle_generation: 0,
        actual_generation: 1,
    };
    assert_eq!(stale.value(&runtime), Err(expected_error.clone()));
    assert_eq!(stale.state(&runtime), Err(expected_error.clone()));
    assert_eq!(
        stale.transition_to(3.0, &mut runtime),
        Err(expected_error.clone())
    );
    assert_eq!(stale.pause(&mut runtime), Err(expected_error.clone()));
    assert_eq!(stale.remove(&mut runtime), Err(expected_error));
    assert_approx_eq!(f32, current.value(&runtime).unwrap(), 2.0);
}

#[test]
fn motion_errors_diagnose_cross_runtime_handle_misuse() {
    let mut source_runtime = MotionRuntime::new();
    let float_motion = source_runtime.motion(1.0_f32);
    let empty_runtime = MotionRuntime::new();

    assert_eq!(
        float_motion.value(&empty_runtime),
        Err(MotionError::SlotOutOfBounds { slot: 0 })
    );

    let mut integer_runtime = MotionRuntime::new();
    let _integer_motion = integer_runtime.motion(1_i32);
    assert_eq!(
        float_motion.value(&integer_runtime),
        Err(MotionError::TypeMismatch {
            expected: std::any::type_name::<f32>(),
            actual: std::any::type_name::<i32>(),
        })
    );
}

#[test]
fn shrink_to_fit_releases_unused_slot_capacity() {
    let mut runtime = MotionRuntime::new();
    let motions = (0_u8..16)
        .map(|value| runtime.motion(f32::from(value)))
        .collect::<Vec<_>>();
    let capacity_before = runtime.slot_capacity();

    for motion in motions {
        assert!(runtime.remove(motion).is_ok());
    }
    runtime.shrink_to_fit();

    assert_eq!(runtime.motion_count(), 0);
    assert!(runtime.slot_capacity() <= capacity_before);
}

#[test]
fn drop_when_settled_removes_precompleted_animation_immediately() {
    let mut animation = Tween::between(0.0_f32, 1.0, Timing::new(10.0));
    animation.finish();
    let mut runtime = MotionRuntime::new();

    let motion =
        runtime.insert_with_policy(animation, Timing::default(), RetainPolicy::DropWhenSettled);

    assert_eq!(
        motion.value(&runtime),
        Err(MotionError::Removed { slot: 0 })
    );
    assert_eq!(runtime.motion_count(), 0);
}

#[test]
fn tick_at_uses_elapsed_instants() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion_with(0.0_f32, Timing::new(100.0));
    assert!(motion.transition_to(10.0, &mut runtime).is_ok());
    let start = std::time::Instant::now();

    runtime.tick_at(start);
    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 0.0);
    runtime.tick_at(start + std::time::Duration::from_millis(50));
    assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 5.0, epsilon = 0.001);
}

#[test]
fn presence_accepts_custom_enter_and_exit_animations() {
    let mut runtime = MotionRuntime::new();
    let mut presence = Presence::new(&mut runtime, 0.0_f32, 1.0, Timing::new(100.0));

    presence
        .show_with(Tween::between(0.0, 2.0, Timing::new(50.0)), &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, *presence.value(&runtime).unwrap(), 2.0);

    presence
        .hide_with(Tween::between(2.0, -1.0, Timing::new(50.0)), &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(50.0));
    presence.sync(&runtime).unwrap();

    assert!(!presence.is_mounted());
    assert_approx_eq!(f32, *presence.value(&runtime).unwrap(), -1.0);
}

#[test]
fn spring_lifecycle_commands_preserve_or_update_value() {
    let mut spring = Spring::new(0.0_f32, 10.0, SpringConfig::default());

    spring.pause();
    spring.tick(Duration::from_millis(16.0));
    assert_eq!(spring.state(), AnimationState::Paused);
    assert_approx_eq!(f32, *spring.value(), 0.0);

    spring.resume();
    spring.tick(Duration::from_millis(16.0));
    assert_eq!(spring.state(), AnimationState::Running);
    assert!(*spring.value() > 0.0);

    spring.cancel();
    let canceled_value = *spring.value();
    spring.tick(Duration::from_millis(16.0));
    assert_eq!(spring.state(), AnimationState::Canceled);
    assert_approx_eq!(f32, *spring.value(), canceled_value);
}
