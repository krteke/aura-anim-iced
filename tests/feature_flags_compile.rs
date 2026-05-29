//! Compile coverage for public feature combinations.

use aura_anim_iced::{AnimationRuntime, TickPolicy, timing::Duration};

#[test]
fn default_feature_set_keeps_runtime_api_available() {
    let mut runtime = AnimationRuntime::testing();
    let policy = TickPolicy::new(Duration::from_millis(33.0));

    runtime.set_motion_policy(policy);

    assert_eq!(runtime.motion_policy(), policy);
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
