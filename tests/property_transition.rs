//! Public API coverage for property-driven value change animation.

use aura_anim_iced::{
    behavior::{BehaviorRule, PropertyTransition},
    property::{OPACITY, PropertySnapshot, PropertySpec, PropertyValue, WIDTH},
    runtime::{AnimationRuntime, AnimationTargetId},
    timing::{Duration, Timing},
};
use float_cmp::assert_approx_eq;

fn opacity(snapshot: &PropertySnapshot) -> f32 {
    let Some(entry) = snapshot.find_property(&OPACITY.raw()) else {
        panic!("expected opacity property");
    };
    let PropertyValue::Scalar(value) = entry.value() else {
        panic!("expected scalar value");
    };

    *value
}

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

#[test]
fn property_transition_registers_animation_after_value_change() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.25).is_none());
    assert_eq!(transition.current_value(), Some(0.25));
    assert!(runtime.is_idle());

    assert!(transition.transition_to(&mut runtime, 0.25).is_none());
    assert!(runtime.is_idle());

    let registration = transition
        .transition_to(&mut runtime, 1.0)
        .expect("changed value registers animation");

    assert_eq!(transition.current_value(), Some(1.0));
    assert_eq!(
        registration.registration().properties().map(opacity),
        Some(0.25)
    );
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let mid_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(mid_tick.properties_for(target).expect("target output")),
        0.625,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[registration.handle()]);
    assert_approx_eq!(
        f32,
        opacity(final_tick.properties_for(target).expect("target output")),
        1.0,
        epsilon = 1e-5
    );
    assert!(runtime.is_idle());
}

#[test]
fn tracked_value_changes_start_automatic_transition_from_previous_value() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, WIDTH).with_timing(Timing::new(120.0));

    assert!(transition.transition_to(&mut runtime, 120.0).is_none());
    assert_eq!(transition.current_value(), Some(120.0));
    assert!(runtime.is_idle());

    let registration = transition
        .transition_to(&mut runtime, 300.0)
        .expect("changed tracked value starts an automatic transition");

    assert_eq!(transition.current_value(), Some(300.0));
    assert_eq!(transition.active_handle(), Some(registration.handle()));
    assert_eq!(
        registration
            .registration()
            .properties()
            .map(|properties| scalar(properties, WIDTH)),
        Some(120.0)
    );

    runtime.clock_mut().set_now(Duration::from_millis(60.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        scalar(tick.properties_for(target).expect("width output"), WIDTH),
        210.0,
        epsilon = 1e-5
    );
}

#[test]
fn behavior_rule_can_be_reused_for_multiple_targets() {
    let mut runtime = AnimationRuntime::testing();
    let first = AnimationTargetId::new();
    let second = AnimationTargetId::new();
    let rule = BehaviorRule::new(OPACITY).with_timing(Timing::new(80.0));
    let mut first_transition = rule.bind(first);
    let mut second_transition = rule.bind(second);

    assert_eq!(rule.property(), OPACITY);
    assert_eq!(rule.timing(), Timing::new(80.0));
    assert!(first_transition.transition_to(&mut runtime, 0.0).is_none());
    assert!(second_transition.transition_to(&mut runtime, 0.5).is_none());

    let first_registration = first_transition
        .transition_to(&mut runtime, 1.0)
        .expect("first target changed");
    let second_registration = second_transition
        .transition_to(&mut runtime, 1.0)
        .expect("second target changed");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(first).expect("first target output")),
        0.5,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(second).expect("second target output")),
        0.75,
        epsilon = 1e-5
    );
    assert!(tick.completed().is_empty());

    runtime.clock_mut().set_now(Duration::from_millis(80.0));
    let final_tick = runtime.tick();

    assert_eq!(
        final_tick.completed(),
        &[first_registration.handle(), second_registration.handle()]
    );
}

#[test]
fn behavior_width_example_flow_animates_changing_width_value() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let rule = BehaviorRule::new(WIDTH).with_timing(Timing::new(420.0));
    let mut transition = rule.bind(target);

    assert!(transition.transition_to(&mut runtime, 160.0).is_none());

    let registration = transition
        .transition_to(&mut runtime, 340.0)
        .expect("width value change starts animation");

    assert_eq!(
        registration
            .registration()
            .properties()
            .map(|properties| scalar(properties, WIDTH)),
        Some(160.0)
    );

    runtime.clock_mut().set_now(Duration::from_millis(210.0));
    let tick = runtime.tick();

    assert_eq!(transition.current_value(), Some(340.0));
    assert_approx_eq!(
        f32,
        scalar(tick.properties_for(target).expect("width output"), WIDTH),
        250.0,
        epsilon = 1e-5
    );
}

#[test]
fn behavior_width_controls_can_trigger_repeated_value_changes() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let rule = BehaviorRule::new(WIDTH).with_timing(Timing::new(420.0));
    let mut transition = rule.bind(target);
    let mut visual_width = 90.0;

    assert!(
        transition
            .transition_to(&mut runtime, visual_width)
            .is_none()
    );

    let wide = transition
        .transition_from_visual(&mut runtime, visual_width, 420.0)
        .expect("wide control starts width animation");

    runtime.clock_mut().set_now(Duration::from_millis(210.0));
    let wide_tick = runtime.tick();
    visual_width = scalar(
        wide_tick.properties_for(target).expect("wide output"),
        WIDTH,
    );

    assert_approx_eq!(f32, visual_width, 255.0, epsilon = 1e-5);

    let medium = transition
        .transition_from_visual(&mut runtime, visual_width, 240.0)
        .expect("medium control retargets width animation");
    let active = transition
        .active_transition()
        .expect("active width transition metadata");

    assert_eq!(medium.replaced(), Some(wide.handle()));
    assert_eq!(runtime.active_count(), 1);
    assert_approx_eq!(f32, active.from(), visual_width, epsilon = 1e-5);
    assert_approx_eq!(f32, active.to(), 240.0, epsilon = 1e-5);
    assert_eq!(
        medium
            .registration()
            .properties()
            .map(|properties| scalar(properties, WIDTH)),
        Some(visual_width)
    );
}

#[test]
fn property_transition_continues_from_running_visual_value() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let first_registration = transition
        .transition_to(&mut runtime, 1.0)
        .expect("initial target change");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let first_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(first_tick.properties_for(target).expect("target output")),
        0.4,
        epsilon = 1e-5
    );

    let replacement = transition
        .transition_to(&mut runtime, 0.2)
        .expect("replacement target change");

    assert_ne!(replacement.handle(), first_registration.handle());
    assert_eq!(replacement.replaced(), Some(first_registration.handle()));
    assert_eq!(runtime.active_count(), 1);
    assert_approx_eq!(
        f32,
        opacity(
            replacement
                .registration()
                .properties()
                .expect("replacement output")
        ),
        0.4,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(90.0));
    let continuation_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(
            continuation_tick
                .properties_for(target)
                .expect("target output")
        ),
        0.3,
        epsilon = 1e-5
    );
    assert!(continuation_tick.completed().is_empty());

    runtime.clock_mut().set_now(Duration::from_millis(140.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[replacement.handle()]);
    assert_approx_eq!(
        f32,
        opacity(final_tick.properties_for(target).expect("target output")),
        0.2,
        epsilon = 1e-5
    );
}

#[test]
fn property_transition_can_start_from_explicit_visual_value() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(120.0));

    let registration = transition
        .transition_from_visual(&mut runtime, 0.35, 0.95)
        .expect("explicit visual value starts transition");

    assert_eq!(transition.current_value(), Some(0.95));
    assert_approx_eq!(
        f32,
        opacity(
            registration
                .registration()
                .properties()
                .expect("registration output")
        ),
        0.35,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(60.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(target).expect("target output")),
        0.65,
        epsilon = 1e-5
    );
}

#[test]
fn property_transition_handles_runtime_completion() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let registration = transition
        .transition_to(&mut runtime, 1.0)
        .expect("target change");

    assert_eq!(transition.active_handle(), Some(registration.handle()));
    assert!(transition.is_active(&runtime));
    assert!(!transition.handle_completion(&runtime));

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let final_tick = runtime.tick();

    assert_eq!(final_tick.completed(), &[registration.handle()]);
    assert!(transition.handle_completion(&runtime));
    assert_eq!(transition.active_handle(), None);
    assert!(!transition.is_active(&runtime));
    assert!(!transition.handle_completion(&runtime));
}

#[test]
fn property_transition_retargets_running_animation_to_new_destination() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.retarget_to(&mut runtime, 1.0).is_none());
    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let initial = transition
        .transition_to(&mut runtime, 1.0)
        .expect("initial target change");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(target).expect("target output")),
        0.4,
        epsilon = 1e-5
    );
    assert!(transition.retarget_to(&mut runtime, 1.0).is_none());

    let retargeted = transition
        .retarget_to(&mut runtime, 0.75)
        .expect("running transition retargets");

    assert_ne!(initial.handle(), retargeted.handle());
    assert_eq!(retargeted.replaced(), Some(initial.handle()));
    assert_eq!(transition.current_value(), Some(0.75));
    assert_eq!(transition.active_handle(), Some(retargeted.handle()));
    assert_eq!(runtime.active_count(), 1);
    assert_approx_eq!(
        f32,
        opacity(
            retargeted
                .registration()
                .properties()
                .expect("retarget output")
        ),
        0.4,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(90.0));
    let retarget_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(retarget_tick.properties_for(target).expect("target output")),
        0.575,
        epsilon = 1e-5
    );
}

#[test]
fn retargeting_uses_current_visual_result_instead_of_previous_target() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, WIDTH).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 10.0).is_none());
    let initial = transition
        .transition_to(&mut runtime, 110.0)
        .expect("initial width change starts");

    runtime.clock_mut().set_now(Duration::from_millis(25.0));
    let initial_tick = runtime.tick();
    let visual = scalar(
        initial_tick.properties_for(target).expect("width output"),
        WIDTH,
    );

    assert_approx_eq!(f32, visual, 35.0, epsilon = 1e-5);

    let retargeted = transition
        .retarget_to(&mut runtime, 75.0)
        .expect("active transition retargets");

    assert_eq!(retargeted.replaced(), Some(initial.handle()));
    assert_eq!(
        retargeted
            .registration()
            .properties()
            .map(|properties| scalar(properties, WIDTH)),
        Some(visual)
    );
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(75.0));
    let retarget_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        scalar(
            retarget_tick.properties_for(target).expect("width output"),
            WIDTH
        ),
        55.0,
        epsilon = 1e-5
    );
}

#[test]
fn property_transition_interrupts_running_animation_from_explicit_visual_value() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let initial = transition
        .transition_to(&mut runtime, 1.0)
        .expect("initial target change");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(target).expect("target output")),
        0.4,
        epsilon = 1e-5
    );

    let interrupted = transition
        .interrupt_from_visual(&mut runtime, 0.4, 1.0)
        .expect("same-target interruption restarts from visual value");

    assert_ne!(initial.handle(), interrupted.handle());
    assert_eq!(interrupted.replaced(), Some(initial.handle()));
    assert_eq!(transition.current_value(), Some(1.0));
    assert_eq!(transition.active_handle(), Some(interrupted.handle()));
    assert_eq!(runtime.active_count(), 1);
    assert_approx_eq!(
        f32,
        opacity(
            interrupted
                .registration()
                .properties()
                .expect("interrupted output")
        ),
        0.4,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(90.0));
    let interrupted_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(
            interrupted_tick
                .properties_for(target)
                .expect("target output")
        ),
        0.7,
        epsilon = 1e-5
    );
}

#[test]
fn property_transition_cleans_interrupted_animation_after_replacement_starts() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let initial = transition
        .transition_to(&mut runtime, 1.0)
        .expect("initial target change");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    runtime.tick();

    let interrupted = transition
        .interrupt_from_visual(&mut runtime, 0.4, 1.0)
        .expect("same-target interruption restarts from visual value");

    assert_eq!(interrupted.replaced(), Some(initial.handle()));
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(100.0));
    let original_end_tick = runtime.tick();

    assert!(original_end_tick.completed().is_empty());
    assert_eq!(runtime.active_count(), 1);
    assert_approx_eq!(
        f32,
        opacity(
            original_end_tick
                .properties_for(target)
                .expect("replacement output")
        ),
        0.76,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(140.0));
    let replacement_end_tick = runtime.tick();

    assert_eq!(replacement_end_tick.completed(), &[interrupted.handle()]);
    assert!(runtime.is_idle());
}

#[test]
fn property_transition_tracks_progress_after_direction_change() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let mut transition = PropertyTransition::new(target, OPACITY).with_timing(Timing::new(100.0));

    assert!(transition.transition_to(&mut runtime, 0.0).is_none());
    let forward = transition
        .transition_to(&mut runtime, 1.0)
        .expect("forward transition starts");

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    let forward_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(forward_tick.properties_for(target).expect("target output")),
        0.4,
        epsilon = 1e-5
    );

    let reverse = transition
        .retarget_to(&mut runtime, 0.0)
        .expect("reverse retarget starts");
    let active = transition
        .active_transition()
        .expect("active property transition");

    assert_eq!(reverse.replaced(), Some(forward.handle()));
    assert_eq!(active.handle(), reverse.handle());
    assert_approx_eq!(f32, active.from(), 0.4, epsilon = 1e-5);
    assert_approx_eq!(f32, active.to(), 0.0, epsilon = 1e-5);
    assert_eq!(active.started_at(), Duration::from_millis(40.0));
    assert_eq!(active.duration(), Some(Duration::from_millis(100.0)));

    let start_progress = transition
        .active_progress_at(Duration::from_millis(40.0))
        .expect("reverse start progress");

    assert_eq!(start_progress.progress(), Some(0.0));
    assert_approx_eq!(f32, start_progress.from(), 0.4, epsilon = 1e-5);
    assert_approx_eq!(f32, start_progress.to(), 0.0, epsilon = 1e-5);

    runtime.clock_mut().set_now(Duration::from_millis(90.0));
    let reverse_tick = runtime.tick();
    let reverse_progress = transition
        .active_progress_at(reverse_tick.timestamp())
        .expect("reverse progress");

    assert_eq!(reverse_progress.elapsed(), Duration::from_millis(50.0));
    assert_eq!(reverse_progress.progress(), Some(0.5));
    assert_approx_eq!(
        f32,
        opacity(reverse_tick.properties_for(target).expect("target output")),
        0.2,
        epsilon = 1e-5
    );
}
