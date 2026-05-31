//! Public API coverage for route-driven screen transition animation.

use aura_anim_iced::{
    ActiveRouteTransition, AnimationRuntime, AnimationTargetId, Duration, OPACITY,
    PropertySnapshot, PropertyValue, RouteAnimator, RouteIncomingMotion, RouteScreenTargets,
    RouteScreenTransition, RouteTransition, RouteTransitionSet, TRANSLATE, Timeline, Track,
};
use float_cmp::assert_approx_eq;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    Home,
    Settings,
    Details,
}

fn opacity_timeline(from: f32, to: f32, duration_ms: f64) -> Timeline {
    Timeline::track(
        Track::from(OPACITY, from)
            .to(to)
            .duration(Duration::from_millis(duration_ms)),
    )
}

fn opacity(snapshot: &PropertySnapshot) -> f32 {
    let Some(entry) = snapshot.find_property(&OPACITY.raw()) else {
        panic!("expected opacity property");
    };
    let PropertyValue::Scalar(value) = entry.value() else {
        panic!("expected scalar value");
    };

    *value
}

fn translate(snapshot: &PropertySnapshot) -> iced::Vector {
    let Some(entry) = snapshot.find_property(&TRANSLATE.raw()) else {
        panic!("expected translate property");
    };
    let PropertyValue::Vector2(value) = entry.value() else {
        panic!("expected vector value");
    };

    *value
}

#[test]
fn route_animator_registers_timeline_for_screen_switch() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transition = RouteTransition::new(
        Route::Home,
        Route::Settings,
        opacity_timeline(0.0, 1.0, 120.0),
    );
    let mut animator = RouteAnimator::new(target, Route::Home);

    assert_eq!(transition.from(), Route::Home);
    assert_eq!(transition.to(), Route::Settings);
    assert_eq!(animator.target(), target);
    assert_eq!(animator.current(), Route::Home);

    let registration = animator
        .transition_with(&mut runtime, &transition)
        .expect("route transition starts");
    let active = animator
        .active_transition()
        .expect("active route transition metadata");

    assert_eq!(animator.current(), Route::Settings);
    assert_eq!(animator.active_handle(), Some(registration.handle()));
    assert_eq!(active.handle(), registration.handle());
    assert_eq!(active.from(), Route::Home);
    assert_eq!(active.to(), Route::Settings);
    assert_eq!(active.started_at(), Duration::ZERO);
    assert_eq!(active.duration(), Some(Duration::from_millis(120.0)));
    assert!(animator.is_active(&runtime));
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(60.0));
    let tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(tick.properties_for(target).expect("route output")),
        0.5,
        epsilon = 1e-5
    );
}

#[test]
fn route_transition_set_matches_screen_switches_and_uses_fallback() {
    let mut runtime = AnimationRuntime::testing();
    let target = AnimationTargetId::new();
    let transitions = RouteTransitionSet::from_transitions([RouteTransition::new(
        Route::Home,
        Route::Settings,
        opacity_timeline(0.0, 1.0, 100.0),
    )])
    .with_fallback(opacity_timeline(0.2, 0.8, 200.0));
    let mut animator = RouteAnimator::new(target, Route::Home);

    assert_eq!(transitions.transitions().len(), 1);
    assert!(transitions.find(Route::Home, Route::Settings).is_some());
    assert!(transitions.find(Route::Settings, Route::Details).is_none());
    assert!(
        animator
            .transition_to(&mut runtime, Route::Home, &transitions)
            .is_none()
    );

    let settings = animator
        .transition_to(&mut runtime, Route::Settings, &transitions)
        .expect("matched route transition starts");

    assert_eq!(animator.current(), Route::Settings);
    assert_eq!(animator.active_handle(), Some(settings.handle()));

    runtime.clock_mut().set_now(Duration::from_millis(40.0));
    runtime.tick();

    let details = animator
        .transition_to(&mut runtime, Route::Details, &transitions)
        .expect("fallback route transition starts");

    assert_eq!(
        details.replaced().map(ActiveRouteTransition::handle),
        Some(settings.handle())
    );
    assert_eq!(animator.current(), Route::Details);
    assert_eq!(runtime.active_count(), 1);

    runtime.clock_mut().set_now(Duration::from_millis(140.0));
    let fallback_tick = runtime.tick();

    assert_approx_eq!(
        f32,
        opacity(
            fallback_tick
                .properties_for(target)
                .expect("fallback output")
        ),
        0.5,
        epsilon = 1e-5
    );
}

#[test]
fn route_screen_transition_runs_outgoing_before_incoming_reaches_final_state() {
    let mut runtime = AnimationRuntime::testing();
    let route_target = AnimationTargetId::new();
    let outgoing_target = AnimationTargetId::new();
    let incoming_target = AnimationTargetId::new();
    let transition = RouteScreenTransition::new(
        Route::Home,
        Route::Settings,
        opacity_timeline(1.0, 0.0, 80.0),
        opacity_timeline(0.0, 1.0, 140.0),
    );
    let mut animator = RouteAnimator::new(route_target, Route::Home);

    assert_eq!(transition.from(), Route::Home);
    assert_eq!(transition.to(), Route::Settings);
    assert_eq!(
        transition.total_duration(),
        Some(Duration::from_millis(140.0))
    );
    assert_eq!(transition.outgoing_shorter(), Some(true));

    let registration = animator
        .transition_screens_with(
            &mut runtime,
            &transition,
            RouteScreenTargets::new(outgoing_target, incoming_target),
        )
        .expect("screen transition starts");

    assert_eq!(animator.current(), Route::Settings);
    assert_eq!(
        animator.active_handle(),
        Some(registration.route().handle())
    );
    assert_eq!(runtime.active_count(), 3);
    assert_approx_eq!(
        f32,
        opacity(
            registration
                .outgoing()
                .properties()
                .expect("outgoing initial output")
        ),
        1.0,
        epsilon = 1e-5
    );
    assert_approx_eq!(
        f32,
        opacity(
            registration
                .incoming()
                .properties()
                .expect("incoming initial output")
        ),
        0.0,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(80.0));
    let outgoing_end_tick = runtime.tick();

    assert_eq!(
        outgoing_end_tick.completed(),
        &[registration.outgoing().handle()]
    );
    assert!(animator.is_active(&runtime));
    assert_approx_eq!(
        f32,
        opacity(
            outgoing_end_tick
                .properties_for(incoming_target)
                .expect("incoming output")
        ),
        80.0 / 140.0,
        epsilon = 1e-5
    );

    runtime.clock_mut().set_now(Duration::from_millis(140.0));
    let incoming_end_tick = runtime.tick();

    assert!(
        incoming_end_tick
            .completed()
            .contains(&registration.route().handle())
    );
    assert!(
        incoming_end_tick
            .completed()
            .contains(&registration.incoming().handle())
    );
    assert!(
        !incoming_end_tick
            .completed()
            .contains(&registration.outgoing().handle())
    );
}

#[test]
fn route_screen_transition_can_build_incoming_opacity_and_position_motion() {
    let mut runtime = AnimationRuntime::testing();
    let route_target = AnimationTargetId::new();
    let outgoing_target = AnimationTargetId::new();
    let incoming_target = AnimationTargetId::new();
    let incoming =
        RouteIncomingMotion::new(iced::Vector::new(24.0, 8.0), Duration::from_millis(120.0));
    let transition = RouteScreenTransition::with_incoming_motion(
        Route::Home,
        Route::Details,
        opacity_timeline(1.0, 0.0, 80.0),
        incoming,
    );
    let mut animator = RouteAnimator::new(route_target, Route::Home);

    assert_eq!(incoming.offset(), iced::Vector::new(24.0, 8.0));
    assert_eq!(incoming.duration(), Duration::from_millis(120.0));

    let registration = animator
        .transition_screens_with(
            &mut runtime,
            &transition,
            RouteScreenTargets::new(outgoing_target, incoming_target),
        )
        .expect("screen transition starts");
    let initial = registration
        .incoming()
        .properties()
        .expect("incoming initial output");

    assert_approx_eq!(f32, opacity(initial), 0.0, epsilon = 1e-5);
    assert_eq!(translate(initial), iced::Vector::new(24.0, 8.0));

    runtime.clock_mut().set_now(Duration::from_millis(60.0));
    let mid_tick = runtime.tick();
    let mid = mid_tick
        .properties_for(incoming_target)
        .expect("incoming mid output");

    assert_approx_eq!(f32, opacity(mid), 0.5, epsilon = 1e-5);
    assert_eq!(translate(mid), iced::Vector::new(12.0, 4.0));

    runtime.clock_mut().set_now(Duration::from_millis(120.0));
    let final_tick = runtime.tick();
    let final_snapshot = final_tick
        .properties_for(incoming_target)
        .expect("incoming final output");

    assert_approx_eq!(f32, opacity(final_snapshot), 1.0, epsilon = 1e-5);
    assert_eq!(translate(final_snapshot), iced::Vector::new(0.0, 0.0));
    assert!(
        final_tick
            .completed()
            .contains(&registration.incoming().handle())
    );
}
