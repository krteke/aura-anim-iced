//! Core animation primitives, timing types, composition utilities, and runtime storage.

pub mod binding;
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

pub use aura_anim_macros::Animatable;
pub use binding::{MotionBinding, MotionBindingError, MotionBindingState, TransitionContext};
pub use interpolate::InterpolationProgress;
pub use presence::Presence;
pub use runtime::{AnimationCommand, Motion, MotionError, MotionRuntime, RetainPolicy};
pub use spring::{Spring, SpringConfig};
pub use timeline::{Hold, Parallel, Sequence, Timeline};
pub use traits::{Animatable, Animation, AnimationExt, AnimationState, BoxAnimation, Interpolate};
pub use tween::{Tween, TweenState};
