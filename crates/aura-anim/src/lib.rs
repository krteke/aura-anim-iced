pub use aura_anim_core as core;
pub use aura_anim_iced as iced;

pub use aura_anim_core::{
    Animatable, Animation, AnimationCommand, AnimationState, Hold, Interpolate,
    InterpolationProgress, Motion, MotionRuntime, Parallel, Presence, RetainPolicy, Sequence,
    Spring, SpringConfig, Timeline, Tween, TweenState, keyframes::Keyframes,
};

pub mod prelude {
    pub use aura_anim_core::{
        Animatable, Animation, AnimationCommand, AnimationState, Hold, Interpolate,
        InterpolationProgress, Motion, MotionRuntime, Parallel, Presence, RetainPolicy, Sequence,
        Spring, SpringConfig, Timeline, Tween, TweenState,
        keyframes::Keyframes,
        timing::{Delay, Direction, Duration, Easing, IterationCount, Timing},
    };
    pub use aura_anim_iced::{TickPolicy, subscription, subscription_with_policy};
}
