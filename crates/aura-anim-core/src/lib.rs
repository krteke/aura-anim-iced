//! Core animation primitives, timing types, composition utilities, and runtime storage.

pub mod binding;
pub mod field;
#[cfg(feature = "iced")]
mod iced;
pub mod interpolate;
pub mod keyframes;
pub mod presence;
pub mod runtime;
pub mod spring;
pub mod timeline;
pub mod timing;
pub mod traits;
pub mod tween;

pub use aura_anim_macros::{Animatable, field};
pub use binding::{MotionBinding, MotionBindingError, MotionBindingState, TransitionContext};
pub use field::{Field, Fields, FieldsAnimation, fields};
pub use interpolate::InterpolationProgress;
pub use presence::Presence;
pub use runtime::{
    AnimationCommand, InterruptionReason, Motion, MotionError, MotionEvent, MotionEventKind,
    MotionEventTarget, MotionId, MotionRuntime, PlaybackId, RemovalReason, RetainPolicy,
};
pub use spring::{Spring, SpringConfig};
pub use timeline::{Hold, Parallel, Sequence, Timeline};
pub use traits::{
    Animatable, Animation, AnimationExt, AnimationState, BoxAnimation, Interpolate,
    IntoMotionAnimation,
};
pub use tween::{Tween, TweenState};
