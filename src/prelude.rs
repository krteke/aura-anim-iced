//! Common imports for the v0.1 public API.
//!
//! # Example
//!
//! ```
//! use aura_anim_iced::prelude::*;
//!
//! let target = AnimationTargetId::new();
//! let keyframes = KeyframesBuilder::new()
//!     .with_timing(Timing::new(120.0))
//!     .at(0.0, (OPACITY, 0.0))
//!     .at(1.0, (OPACITY, 1.0))
//!     .finish();
//! let timeline = Timeline::keyframes(keyframes);
//!
//! assert!(timeline.sample_at(Duration::from_millis(60.0)).is_some());
//! assert_ne!(target, AnimationTargetId::new());
//! ```

pub use crate::behavior::{
    ActivePropertyTransition, BehaviorRule, PropertyTransition, PropertyTransitionProgress,
    PropertyTransitionRegistration, TransitionValueKind,
};
pub use crate::iced_ext::{EffectSnapshot, effect_snapshot, tick_effect_snapshot_for};
pub use crate::keyframes::{Keyframe, Keyframes, KeyframesBuilder};
pub use crate::property::{
    BACKGROUND, BORDER_COLOR, Color as ColorProperty, HEIGHT, OPACITY, PADDING, PropertyEntry,
    PropertyKey, PropertySnapshot, PropertySpec, PropertyValue, PropertyValueKind, RADIUS,
    RawPropertySpec, Rectangle as RectangleProperty, SCALE, SHADOW, Scalar as ScalarProperty,
    Shadow as ShadowProperty, Size as SizeProperty, TEXT_COLOR, TRANSLATE, TransformValue,
    Vector2 as Vector2Property, WIDTH,
};
pub use crate::route::{
    ActiveRouteTransition, RouteAnimator, RouteIncomingMotion, RouteScreenTargets,
    RouteScreenTransition, RouteScreenTransitionRegistration, RouteTransition,
    RouteTransitionRegistration, RouteTransitionSet,
};
pub use crate::runtime::{
    AnimationHandle, AnimationPlaybackState, AnimationRegistration, AnimationRuntime,
    AnimationTargetId, AnimationTick, TargetedPropertySnapshot, TickPolicy,
};
pub use crate::state::{
    ActiveStateTransition, StateAnimator, StateTransition, StateTransitionProgress,
    StateTransitionRegistration, StateTransitionSet,
};
pub use crate::timeline::{
    Hold, Parallel, PropertyTrackBuilder, Sequence, Timeline, TimelineMarker, TimelineStep, Track,
};
pub use crate::timing::{
    Delay, Direction, Duration, Easing, FillMode, IterationCount, NormalizedTiming, Timing,
    TimingPhase, TimingSampleState,
};
