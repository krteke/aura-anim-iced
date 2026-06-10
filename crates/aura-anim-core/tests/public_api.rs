//! Integration tests for the public aura-anim-core API.

use aura_anim_core::{
    Animatable, Animation, AnimationCommand, AnimationState, Hold, Interpolate, InterruptionReason,
    MotionBinding, MotionError, MotionEventKind, MotionRuntime, Parallel, Presence, RemovalReason,
    RetainPolicy, Sequence, Spring, SpringConfig, Timeline, Tween, field, fields,
    keyframes::{Keyframe, Keyframes},
    spring_to,
    timing::{Delay, Direction, Duration, Easing, IterationCount, Timing},
    tween_to,
};
use float_cmp::assert_approx_eq;

#[derive(Clone, Debug, Animatable)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone, Debug, Animatable)]
struct Offset(f32, f32);

#[derive(Clone, Debug, Animatable)]
struct GenericValue<T> {
    value: T,
}

#[derive(Clone, Debug, Animatable)]
#[animatable(fields = CustomPositionFields)]
struct CustomPosition {
    value: f32,
}

fn assert_position(value: &Position, x: f32, y: f32) {
    assert_approx_eq!(f32, value.x, x, epsilon = 0.000_1);
    assert_approx_eq!(f32, value.y, y, epsilon = 0.000_1);
}

#[test]
fn derive_generates_named_tuple_generic_and_custom_field_descriptors() {
    let named = PositionFields::x;
    let named_macro = field!(Position::y);
    let tuple = OffsetFields::_0;
    let tuple_macro = field!(Offset::1);
    let generic = GenericValueFields::<f32>::value;
    let generic_macro = field!(GenericValue<f32>::value);
    let custom = CustomPositionFields::value;

    assert_eq!(named.name(), "x");
    assert_eq!(named_macro.name(), "y");
    assert_eq!(tuple.name(), "0");
    assert_eq!(tuple_macro.name(), "1");
    assert_eq!(generic.name(), "value");
    assert_eq!(generic_macro.name(), "value");
    assert_eq!(custom.name(), "value");
}

#[test]
fn motion_plays_fields_with_independent_timings() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion(Position { x: 0.0, y: 0.0 });

    motion
        .play(
            fields()
                .animate(PositionFields::x, |from| {
                    Tween::between(from, 100.0, Timing::new(100.0).with_easing(Easing::EaseIn))
                })
                .animate(field!(Position::y), |from| {
                    Tween::between(from, 200.0, Timing::new(200.0).with_easing(Easing::EaseOut))
                }),
            &mut runtime,
        )
        .unwrap();

    runtime.tick(Duration::from_millis(100.0));
    let halfway = motion.value(&runtime).unwrap();
    assert_approx_eq!(f32, halfway.x, 100.0, epsilon = 0.000_1);
    assert!(halfway.y > 100.0 && halfway.y < 200.0);
    assert_eq!(motion.state(&runtime), Ok(AnimationState::Running));

    runtime.tick(Duration::from_millis(100.0));
    assert_position(&motion.value(&runtime).unwrap(), 100.0, 200.0);
    assert_eq!(motion.state(&runtime), Ok(AnimationState::Completed));
}

#[test]
fn target_factories_drive_motions_and_fields_from_current_values() {
    let mut runtime = MotionRuntime::new();
    let scalar = runtime.motion(0.0_f32);

    scalar
        .play(tween_to(100.0, Timing::linear(100.0)), &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(40.0));
    assert_approx_eq!(f32, scalar.value(&runtime).unwrap(), 40.0);

    scalar
        .play(tween_to(200.0, Timing::linear(100.0)), &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, scalar.value(&runtime).unwrap(), 120.0);

    scalar
        .play(spring_to(300.0, SpringConfig::snappy()), &mut runtime)
        .unwrap();
    assert_approx_eq!(f32, scalar.value(&runtime).unwrap(), 120.0);
    scalar.finish(&mut runtime).unwrap();
    assert_approx_eq!(f32, scalar.value(&runtime).unwrap(), 300.0);

    let position = runtime.motion(Position { x: 10.0, y: 20.0 });
    position
        .play(
            fields()
                .animate(field!(Position::x), tween_to(110.0, Timing::linear(100.0)))
                .animate(
                    field!(Position::y),
                    spring_to(220.0, SpringConfig::snappy()),
                ),
            &mut runtime,
        )
        .unwrap();
    runtime.tick(Duration::from_millis(50.0));

    let halfway = position.value(&runtime).unwrap();
    assert_approx_eq!(f32, halfway.x, 60.0);
    assert!(halfway.y > 20.0);
    position.finish(&mut runtime).unwrap();
    assert_position(&position.value(&runtime).unwrap(), 110.0, 220.0);
}

#[test]
fn interrupted_field_playback_starts_from_the_current_sample() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion(Position { x: 0.0, y: 25.0 });

    motion
        .play(
            fields().animate(field!(Position::x), |from| {
                Tween::between(from, 100.0, Timing::new(100.0))
            }),
            &mut runtime,
        )
        .unwrap();
    runtime.tick(Duration::from_millis(40.0));
    assert_position(&motion.value(&runtime).unwrap(), 40.0, 25.0);

    motion
        .play(
            fields().animate(PositionFields::x, |from| {
                Tween::between(from, 200.0, Timing::new(100.0))
            }),
            &mut runtime,
        )
        .unwrap();
    runtime.tick(Duration::from_millis(50.0));

    assert_position(&motion.value(&runtime).unwrap(), 120.0, 25.0);
}

#[test]
fn field_plan_accepts_different_animation_types() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion(Position { x: 0.0, y: 0.0 });

    motion
        .play(
            fields()
                .animate(PositionFields::x, |from| {
                    Tween::between(from, 100.0, Timing::new(100.0))
                })
                .animate(PositionFields::y, |from| {
                    Spring::new(from, 200.0, SpringConfig::snappy())
                }),
            &mut runtime,
        )
        .unwrap();

    motion.finish(&mut runtime).unwrap();

    assert_position(&motion.value(&runtime).unwrap(), 100.0, 200.0);
    assert_eq!(motion.state(&runtime), Ok(AnimationState::Completed));
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
            context.tween(Timing::new(100.0))
        })
        .fallback(|context| context.tween(Timing::new(50.0)));
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
fn motion_binding_tracked_state_changes_match_runtime_events() {
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

    let active_playback = binding
        .set_state_tracked(&mut state, State::Active, motion, &mut runtime)
        .unwrap()
        .unwrap();
    assert_eq!(
        binding
            .set_state_tracked(&mut state, State::Active, motion, &mut runtime)
            .unwrap(),
        None
    );

    let idle_playback = binding
        .set_state_tracked(&mut state, State::Idle, motion, &mut runtime)
        .unwrap()
        .unwrap();
    let interrupted = runtime.take_events().pop().unwrap();

    assert!(interrupted.is_for(active_playback));
    assert_eq!(
        interrupted.kind(),
        MotionEventKind::Interrupted(InterruptionReason::Replaced)
    );

    runtime.tick(Duration::from_millis(20.0));
    let completed = runtime.take_events().pop().unwrap();

    assert!(completed.is_completed_for(idle_playback));
    assert_eq!(state.current(), &State::Idle);
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
fn timing_easing_constructors_preserve_composition() {
    let linear = Timing::linear(100.0);
    let ease_in = Timing::ease_in(120.0);
    let ease_out = Timing::ease_out(140.0);
    let ease_in_out = Timing::ease_in_out(160.0)
        .with_delay(Delay::from_millis(20.0))
        .with_iterations(2);

    assert_eq!(linear.easing(), Easing::Linear);
    assert_eq!(ease_in.easing(), Easing::EaseIn);
    assert_eq!(ease_out.easing(), Easing::EaseOut);
    assert_eq!(ease_in_out.easing(), Easing::EaseInOut);
    assert_approx_eq!(f64, linear.duration().as_millis(), 100.0);
    assert_approx_eq!(f64, ease_in.duration().as_millis(), 120.0);
    assert_approx_eq!(f64, ease_out.duration().as_millis(), 140.0);
    assert_approx_eq!(
        f64,
        ease_in_out.total_duration().unwrap().as_millis(),
        340.0
    );
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
fn runtime_queues_completion_once_until_events_are_taken() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion(0.0_f32);
    let playback = motion.transition_to_tracked(10.0, &mut runtime).unwrap();

    runtime.tick(Duration::from_millis(200.0));
    runtime.tick(Duration::from_millis(200.0));

    assert_eq!(runtime.pending_event_count(), 1);
    let event = runtime.events()[0];
    assert_eq!(event.kind(), MotionEventKind::Completed);
    assert!(event.is_completed_for(motion));
    assert!(event.is_completed_for(playback));
    assert_eq!(event.motion(), motion.motion_id());
    assert_eq!(event.playback(), playback);

    assert_eq!(runtime.take_events(), vec![event]);
    assert!(runtime.events().is_empty());
}

#[test]
fn tracked_playbacks_distinguish_replacement_and_retarget_events() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion(0.0_f32);
    let first = motion
        .play_tracked(Tween::between(0.0, 10.0, Timing::new(100.0)), &mut runtime)
        .unwrap();
    let second = motion
        .play_tracked(Tween::between(0.0, 20.0, Timing::new(100.0)), &mut runtime)
        .unwrap();
    let third = motion.transition_to_tracked(30.0, &mut runtime).unwrap();

    assert_ne!(first, second);
    assert_ne!(second, third);
    assert_eq!(motion.playback(&runtime), Ok(third));
    let events = runtime.take_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].playback(), first);
    assert_eq!(
        events[0].kind(),
        MotionEventKind::Interrupted(InterruptionReason::Replaced)
    );
    assert_eq!(events[1].playback(), second);
    assert_eq!(
        events[1].kind(),
        MotionEventKind::Interrupted(InterruptionReason::Retargeted)
    );
}

#[test]
fn transition_reports_retargeting_when_source_cannot_retarget_in_place() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.insert(ProgressAnimation::new(), Timing::new(100.0));
    let previous = motion.playback(&runtime).unwrap();

    let next = motion.transition_to_tracked(10.0, &mut runtime).unwrap();

    assert_ne!(previous, next);
    let events = runtime.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].playback(), previous);
    assert_eq!(
        events[0].kind(),
        MotionEventKind::Interrupted(InterruptionReason::Retargeted)
    );
}

#[test]
fn finish_and_cancel_emit_one_terminal_event_per_playback() {
    let mut runtime = MotionRuntime::new();
    let finished = runtime.motion(0.0_f32);
    let canceled = runtime.motion(0.0_f32);
    let finished_playback = finished.transition_to_tracked(1.0, &mut runtime).unwrap();
    let canceled_playback = canceled.transition_to_tracked(1.0, &mut runtime).unwrap();

    finished.finish(&mut runtime).unwrap();
    finished.finish(&mut runtime).unwrap();
    canceled.cancel(&mut runtime).unwrap();
    canceled.cancel(&mut runtime).unwrap();

    let events = runtime.take_events();
    assert_eq!(events.len(), 2);
    assert!(events[0].is_completed_for(finished_playback));
    assert!(events[1].is_canceled_for(canceled_playback));
}

#[test]
fn drop_when_settled_emits_terminal_then_removal_events() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.play_once(Tween::between(0.0_f32, 1.0, Timing::new(10.0)));
    let playback = motion.playback(&runtime).unwrap();

    runtime.tick(Duration::from_millis(10.0));

    let events = runtime.take_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind(), MotionEventKind::Completed);
    assert_eq!(
        events[1].kind(),
        MotionEventKind::Removed(RemovalReason::Settled)
    );
    assert!(events.iter().all(|event| event.is_for(playback)));
    assert_eq!(
        motion.value(&runtime),
        Err(MotionError::Removed { slot: 0 })
    );
}

#[test]
fn removing_a_running_motion_emits_interruption_then_removal() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion(0.0_f32);
    let playback = motion.transition_to_tracked(1.0, &mut runtime).unwrap();

    motion.remove(&mut runtime).unwrap();

    let events = runtime.take_events();
    assert_eq!(events.len(), 2);
    assert_eq!(
        events[0].kind(),
        MotionEventKind::Interrupted(InterruptionReason::Removed)
    );
    assert_eq!(
        events[1].kind(),
        MotionEventKind::Removed(RemovalReason::Explicit)
    );
    assert!(events.iter().all(|event| event.is_for(playback)));
}

#[test]
fn completion_events_can_start_follow_up_playback_after_take() {
    let mut runtime = MotionRuntime::new();
    let motion = runtime.motion(0.0_f32);
    let exit = motion
        .play_tracked(Tween::between(0.0, -1.0, Timing::new(100.0)), &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(100.0));

    let events = runtime.take_events();
    let enter = events
        .iter()
        .find(|event| event.is_completed_for(exit))
        .map(|_| {
            motion
                .play_tracked(Tween::between(-1.0, 1.0, Timing::new(100.0)), &mut runtime)
                .unwrap()
        })
        .unwrap();

    runtime.tick(Duration::from_millis(100.0));
    assert!(
        runtime
            .take_events()
            .iter()
            .any(|event| event.is_completed_for(enter))
    );
}

#[test]
fn events_from_reused_slots_do_not_match_new_motions() {
    let mut runtime = MotionRuntime::new();
    let old = runtime.motion(1.0_f32);
    old.remove(&mut runtime).unwrap();
    let old_event = runtime.take_events()[0];

    let current = runtime.motion(2.0_f32);

    assert_ne!(old.motion_id(), current.motion_id());
    assert!(old_event.is_for(old));
    assert!(!old_event.is_for(current));
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
    assert_eq!(
        runtime
            .take_events()
            .iter()
            .map(|event| event.kind())
            .collect::<Vec<_>>(),
        vec![
            MotionEventKind::Completed,
            MotionEventKind::Removed(RemovalReason::Settled),
        ]
    );
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
        .show_with(tween_to(2.0, Timing::linear(50.0)), &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(50.0));
    assert_approx_eq!(f32, *presence.value(&runtime).unwrap(), 2.0);

    presence
        .hide_with(tween_to(-1.0, Timing::linear(50.0)), &mut runtime)
        .unwrap();
    runtime.tick(Duration::from_millis(50.0));
    presence.sync(&runtime).unwrap();

    assert!(!presence.is_mounted());
    assert_approx_eq!(f32, *presence.value(&runtime).unwrap(), -1.0);
}

#[test]
fn presence_only_unmounts_for_the_current_exit_playback() {
    let mut runtime = MotionRuntime::new();
    let mut presence = Presence::new(&mut runtime, 0.0_f32, 1.0, Timing::new(100.0));

    presence.show(&mut runtime).unwrap();
    runtime.tick(Duration::from_millis(100.0));
    runtime.clear_events();

    presence.hide(&mut runtime).unwrap();
    runtime.tick(Duration::from_millis(100.0));
    let old_exit_events = runtime.take_events();

    presence.show(&mut runtime).unwrap();
    assert!(presence.is_mounted());
    assert!(
        old_exit_events
            .iter()
            .all(|event| !presence.handle_event(event))
    );
    assert!(presence.is_mounted());

    runtime.tick(Duration::from_millis(100.0));
    runtime.clear_events();
    presence.hide(&mut runtime).unwrap();
    runtime.tick(Duration::from_millis(100.0));

    let changed = runtime
        .take_events()
        .iter()
        .any(|event| presence.handle_event(event));
    assert!(changed);
    assert!(!presence.is_mounted());
}

#[test]
fn presence_event_reports_only_actual_mount_changes() {
    let mut runtime = MotionRuntime::new();
    let mut presence = Presence::new(&mut runtime, 0.0_f32, 1.0, Timing::new(100.0));

    presence.hide(&mut runtime).unwrap();
    runtime.tick(Duration::from_millis(100.0));

    let changed = runtime
        .take_events()
        .iter()
        .any(|event| presence.handle_event(event));
    assert!(!changed);
    assert!(!presence.is_mounted());
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
