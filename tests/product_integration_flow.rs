//! Product integration coverage for the standard animation flow.

use std::time::Instant;

use aura_anim_iced::{
    behavior::BehaviorRule,
    iced_ext::AnimationFlow,
    keyframes::KeyframesBuilder,
    property::{OPACITY, PropertySnapshot, PropertySpec, PropertyValue, WIDTH},
    route::{RouteAnimator, RouteIncomingMotion, RouteScreenTargets, RouteScreenTransition},
    runtime::{AnimationRuntime, AnimationTargetId},
    state::{StateAnimator, StateTransition, StateTransitionSet},
    timeline::{Timeline, Track},
    timing::{Duration, Timing},
};
use float_cmp::assert_approx_eq;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Panel {
    Closed,
    Open,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Route {
    Home,
    Settings,
}

fn animation_tick(_: Instant) {}

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

fn fade(from: f32, to: f32) -> Timeline {
    Timeline::track(
        Track::from(OPACITY, from)
            .to(to)
            .duration(Duration::from_millis(100.0)),
    )
}

#[test]
#[allow(clippy::too_many_lines)]
fn flow_routes_value_behavior_state_and_route_through_one_update_path() {
    let mut flow = AnimationFlow::with_runtime(AnimationRuntime::testing());
    let value_target = AnimationTargetId::new();
    let behavior_target = AnimationTargetId::new();
    let state_target = AnimationTargetId::new();
    let route_target = AnimationTargetId::new();
    let outgoing_target = AnimationTargetId::new();
    let incoming_target = AnimationTargetId::new();

    let value = flow.runtime_mut().register_keyframes(
        value_target,
        KeyframesBuilder::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0)
            .finish(),
    );
    flow.capture(&value);

    let mut width = BehaviorRule::new(WIDTH)
        .with_timing(Timing::new(100.0))
        .bind(behavior_target);
    assert!(width.transition_to(flow.runtime_mut(), 100.0).is_none());
    let behavior = width
        .transition_to(flow.runtime_mut(), 200.0)
        .expect("behavior transition");
    flow.capture(&behavior);

    let transitions = StateTransitionSet::from_transitions([StateTransition::new(
        Panel::Closed,
        Panel::Open,
        fade(0.0, 1.0),
    )]);
    let mut panel = StateAnimator::new(state_target, Panel::Closed);
    let state = panel
        .transition_to(flow.runtime_mut(), Panel::Open, &transitions)
        .expect("state transition");
    flow.capture(&state);

    let mut route = RouteAnimator::new(route_target, Route::Home);
    let route_screen = RouteScreenTransition::with_incoming_motion(
        Route::Home,
        Route::Settings,
        Timeline::track(
            Track::from(OPACITY, 1.0)
                .to(0.0)
                .duration(Duration::from_millis(100.0)),
        ),
        RouteIncomingMotion::new(iced::Vector::new(20.0, 0.0), Duration::from_millis(100.0)),
    );
    let screen = route
        .transition_screens_with(
            flow.runtime_mut(),
            &route_screen,
            RouteScreenTargets::new(outgoing_target, incoming_target),
        )
        .expect("route screen transition");
    flow.capture(&screen);

    assert!(flow.should_subscribe());
    let _subscription = flow.subscription(animation_tick);
    assert_eq!(flow.output().properties().targets().count(), 5);
    assert!(flow.output().properties_for(route_target).is_none());

    flow.runtime_mut()
        .clock_mut()
        .set_now(Duration::from_millis(50.0));
    let active = flow.update_tick(Instant::now());
    let active = active.flow().output();

    assert_approx_eq!(
        f32,
        scalar(
            active
                .properties_for(value_target)
                .expect("value target output"),
            OPACITY
        ),
        0.5,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        scalar(
            active
                .properties_for(behavior_target)
                .expect("behavior target output"),
            WIDTH
        ),
        150.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        scalar(
            active
                .properties_for(state_target)
                .expect("state target output"),
            OPACITY
        ),
        0.5,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        flow.target(incoming_target)
            .get(OPACITY)
            .expect("incoming opacity"),
        0.5,
        epsilon = 1e-5
    );
    assert_eq!(
        flow.target(incoming_target)
            .effects()
            .translation
            .expect("incoming translation"),
        iced::Vector::new(10.0, 0.0)
    );

    flow.runtime_mut()
        .clock_mut()
        .set_now(Duration::from_millis(100.0));
    let completed = flow.update_tick(Instant::now()).flow().output().clone();

    assert!(!completed.completed().is_empty());
    assert_eq!(flow.runtime().active_count(), 0);
    assert!(!flow.should_subscribe());
    assert_approx_eq!(
        f32,
        flow.target(value_target)
            .get(OPACITY)
            .expect("completion opacity"),
        1.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        scalar(
            completed
                .properties_for(behavior_target)
                .expect("behavior completion"),
            WIDTH
        ),
        200.0,
        epsilon = 1e-5
    );
    let incoming_completion = flow.target(incoming_target).effects();
    assert_approx_eq!(
        f32,
        incoming_completion.opacity.expect("incoming opacity"),
        1.0,
        epsilon = 1e-5
    );
    assert_eq!(
        incoming_completion
            .translation
            .expect("incoming translation"),
        iced::Vector::new(0.0, 0.0)
    );

    assert!(flow.cleanup_completed(&mut width));
    assert!(flow.cleanup_completed(&mut panel));
    assert!(flow.cleanup_completed(&mut route));
    assert_eq!(width.active_handle(), None);
    assert_eq!(panel.active_handle(), None);
    assert_eq!(route.active_handle(), None);
    assert!(route.active_screen_transition().is_none());
}
