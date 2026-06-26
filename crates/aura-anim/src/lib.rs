//! Convenience facade for the Aura animation core and Iced integration.

pub use aura_anim_core as core;
pub use aura_anim_iced as iced;

/// Common animation and Iced integration imports.
pub mod prelude {
    pub use aura_anim_core::{
        field::fields,
        keyframes::Keyframes,
        macros::{Animatable, field},
        runtime::MotionError,
        runtime::{AnimationCommand, Motion, MotionRuntime},
        spring::{Spring, SpringConfig},
        target::{spring_to, tween_to},
        timeline::{Hold, Parallel, Sequence, Timeline},
        timing::{Delay, Direction, Duration, Easing, IterationCount, Timing},
        traits::{Animation, AnimationExt},
        tween::Tween,
    };

    pub use aura_anim_iced::{TickPolicy, subscription, subscription_with_policy};
}
