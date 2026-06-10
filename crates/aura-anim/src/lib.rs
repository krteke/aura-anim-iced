//! Convenience facade for the Aura animation core and Iced integration.

pub use aura_anim_core as core;
pub use aura_anim_iced as iced;

pub use aura_anim_core::{
    Animatable, Animation, AnimationCommand, AnimationExt, AnimationState, BoxAnimation, Field,
    Fields, FieldsAnimation, Hold, Interpolate, InterpolationProgress, InterruptionReason,
    IntoMotionAnimation, Motion, MotionBinding, MotionBindingError, MotionBindingState,
    MotionError, MotionEvent, MotionEventKind, MotionEventTarget, MotionId, MotionRuntime,
    Parallel, PlaybackId, Presence, RemovalReason, RetainPolicy, Sequence, Spring, SpringConfig,
    Timeline, TransitionContext, Tween, TweenState, field, fields, keyframes::Keyframes, spring_to,
    tween_to,
};

/// Common animation and Iced integration imports.
pub mod prelude {
    pub use aura_anim_core::{
        Animatable, Animation, AnimationCommand, AnimationExt, AnimationState, BoxAnimation, Field,
        Fields, FieldsAnimation, Hold, Interpolate, InterpolationProgress, InterruptionReason,
        IntoMotionAnimation, Motion, MotionBinding, MotionBindingError, MotionBindingState,
        MotionError, MotionEvent, MotionEventKind, MotionEventTarget, MotionId, MotionRuntime,
        Parallel, PlaybackId, Presence, RemovalReason, RetainPolicy, Sequence, Spring,
        SpringConfig, Timeline, TransitionContext, Tween, TweenState, field, fields,
        keyframes::Keyframes,
        spring_to,
        timing::{Delay, Direction, Duration, Easing, IterationCount, Timing},
        tween_to,
    };
    pub use aura_anim_iced::{TickPolicy, subscription, subscription_with_policy};
}
