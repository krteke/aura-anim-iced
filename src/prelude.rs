//! Common imports for product animation code.
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

pub use crate::behavior::{BehaviorRule, PropertyTransition};
#[cfg(feature = "palette")]
pub use crate::color::Oklaba;
pub use crate::color::{AnimColor, Srgba, tag};
pub use crate::defaults::DefaultMotions;
#[cfg(feature = "spring")]
pub use crate::defaults::SpringMotionDefaults;
pub use crate::iced_ext::{
    AnimationCompletionCleanup, AnimationFlow, AnimationTargetOutput, EffectSnapshot,
    target_output_for, tick_effect_snapshot_for,
};
pub use crate::keyframes::{Keyframes, KeyframesBuilder};
pub use crate::property::{
    BACKGROUND, BORDER_COLOR, HEIGHT, OPACITY, PADDING, PropertySnapshot, PropertySpec,
    PropertyValue, RADIUS, SCALE, SHADOW, TEXT_COLOR, TRANSLATE, TransformValue, WIDTH,
};
pub use crate::route::{
    RouteAnimator, RouteIncomingMotion, RouteScreenTargets, RouteScreenTransition, RouteTransition,
    RouteTransitionSet,
};
pub use crate::runtime::{AnimationRuntime, AnimationTargetId, AnimationTick, TickPolicy};
#[cfg(feature = "spring")]
pub use crate::spring::{ScalarSpring, ScalarSpringSample, SpringConfig};
pub use crate::state::{StateAnimator, StateTransition, StateTransitionSet};
pub use crate::timeline::{Hold, Parallel, Sequence, Timeline, TimelineStep, Track};
pub use crate::timing::{Delay, Direction, Duration, Easing, FillMode, Timing};
