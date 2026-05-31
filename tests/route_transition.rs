//! Public API coverage for route-driven screen transition animation.

use aura_anim_iced::{
    ActiveRouteTransition, AnimationRuntime, AnimationTargetId, Duration, OPACITY,
    PropertySnapshot, PropertyValue, RouteAnimator, RouteTransition, RouteTransitionSet, Timeline,
    Track,
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
