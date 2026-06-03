//! Compile coverage for public feature combinations.

use aura_anim_iced::{
    behavior::BehaviorRule,
    iced_ext::AnimationFlow,
    keyframes::KeyframesBuilder,
    property::{OPACITY, WIDTH},
    route::RouteAnimator,
    runtime::{AnimationRuntime, TickPolicy},
    state::StateAnimator,
    timeline::{Timeline, Track},
    timing::{Duration, Timing},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ProbeState {
    Idle,
}

#[test]
fn default_feature_set_keeps_runtime_api_available() {
    let mut runtime = AnimationRuntime::testing();
    let policy = TickPolicy::new(Duration::from_millis(33.0));

    runtime.set_motion_policy(policy);

    assert_eq!(runtime.motion_policy(), policy);
}

#[test]
fn default_feature_set_keeps_core_animation_api_available() {
    let target = aura_anim_iced::runtime::AnimationTargetId::new();
    let keyframes = KeyframesBuilder::new()
        .with_timing(Timing::new(100.0))
        .at(0.0, (OPACITY, 0.0))
        .at(1.0, (OPACITY, 1.0))
        .finish();
    let timeline = Timeline::track(
        Track::from(WIDTH, 0.0)
            .to(100.0)
            .duration(Duration::from_millis(100.0)),
    );
    let behavior = BehaviorRule::new(WIDTH);
    let state = StateAnimator::new(target, ProbeState::Idle);
    let route = RouteAnimator::new(target, ProbeState::Idle);
    let flow = AnimationFlow::with_runtime(AnimationRuntime::testing());

    assert!(keyframes.sample_at(0.5).is_some());
    assert_eq!(
        timeline.total_duration(),
        Some(Duration::from_millis(100.0))
    );
    assert_eq!(behavior.property(), WIDTH);
    assert_eq!(state.current(), ProbeState::Idle);
    assert_eq!(route.current(), ProbeState::Idle);
    assert!(!flow.should_subscribe());
}

#[cfg(feature = "palette")]
#[test]
fn palette_feature_is_available_for_color_interpolation_work() {
    use aura_anim_iced::color::AnimColor;

    let color = AnimColor::oklaba_from_srgba(0.0, 0.0, 0.0, 1.0);
    let sampled = iced::Color::from(color);

    assert_eq!(sampled.a, 1.0);
}

#[cfg(feature = "spring")]
#[test]
fn spring_feature_is_available_for_spring_sampling_work() {
    assert!(cfg!(feature = "spring"));
}

#[cfg(feature = "widgets")]
#[test]
fn widgets_feature_is_available_for_widget_motion_work() {
    assert!(cfg!(feature = "widgets"));
}

#[cfg(feature = "theme")]
#[test]
fn theme_feature_is_available_for_theme_motion_work() {
    assert!(cfg!(feature = "theme"));
}

#[cfg(feature = "layout")]
#[test]
fn layout_feature_is_available_for_layout_motion_work() {
    assert!(cfg!(feature = "layout"));
}

#[cfg(feature = "serde")]
#[test]
fn serde_feature_exposes_serde_for_feature_gated_configuration() {
    #[derive(serde::Deserialize, serde::Serialize)]
    struct FeatureProbe {
        tick_interval_ms: u16,
    }

    let probe = FeatureProbe {
        tick_interval_ms: 16,
    };

    assert_eq!(probe.tick_interval_ms, 16);
}

#[cfg(feature = "tracing")]
#[test]
fn tracing_feature_exposes_runtime_diagnostics_dependency() {
    tracing::trace!(
        target: "aura_anim_iced::runtime",
        feature = "tracing",
        "feature flag compile probe"
    );
}

#[cfg(feature = "inspector")]
#[test]
fn inspector_feature_enables_tracing_backed_diagnostics() {
    tracing::debug!(
        target: "aura_anim_iced::inspector",
        feature = "inspector",
        "inspector feature compile probe"
    );
}
