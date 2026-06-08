//! Iced subscriptions and frame handling for [`aura_anim_core::MotionRuntime`].

use std::time::{Duration, Instant};

use aura_anim_core::MotionRuntime;
use iced::Subscription;

/// Controls how an Iced subscription schedules animation ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TickPolicy {
    /// Request ticks from Iced's window frame stream.
    #[default]
    Frames,
    /// Request ticks at a fixed interval.
    Interval(Duration),
}

impl TickPolicy {
    /// Returns a frame-driven tick policy.
    #[must_use]
    pub const fn frames() -> Self {
        Self::Frames
    }

    /// Returns a fixed-interval tick policy.
    #[must_use]
    pub const fn interval(duration: Duration) -> Self {
        Self::Interval(duration)
    }

    /// Returns a fixed-interval policy for the requested frame rate.
    #[must_use]
    pub fn fps(frames_per_second: u16) -> Self {
        Self::Interval(Duration::from_secs_f64(
            1.0 / f64::from(frames_per_second.max(1)),
        ))
    }
}

/// Creates a frame-driven subscription while the runtime has active animations.
pub fn subscription(runtime: &MotionRuntime) -> Subscription<Instant> {
    subscription_with_policy(runtime, TickPolicy::Frames)
}

/// Creates a subscription using `tick_policy` while the runtime has active animations.
///
/// # Examples
///
/// ```
/// use aura_anim_core::MotionRuntime;
/// use aura_anim_iced::{TickPolicy, subscription_with_policy};
///
/// let runtime = MotionRuntime::new();
/// let subscription = subscription_with_policy(&runtime, TickPolicy::fps(60));
/// # let _ = subscription;
/// ```
pub fn subscription_with_policy(
    runtime: &MotionRuntime,
    tick_policy: TickPolicy,
) -> Subscription<Instant> {
    if runtime.has_active() {
        match tick_policy {
            TickPolicy::Frames => iced::window::frames(),
            TickPolicy::Interval(duration) => {
                iced::time::every(duration.max(Duration::from_millis(1)))
            }
        }
    } else {
        Subscription::none()
    }
}

/// Advances the runtime using an Iced frame timestamp.
pub fn frame(runtime: &mut MotionRuntime, now: Instant) {
    runtime.tick_at(now);
}

/// Common animation and Iced integration imports.
pub mod prelude {
    pub use aura_anim_core::{
        Animatable, Animation, AnimationCommand, AnimationState, Hold, Interpolate,
        InterpolationProgress, Motion, MotionRuntime, Parallel, Presence, RetainPolicy, Sequence,
        Spring, SpringConfig, Timeline, Tween, TweenState,
        keyframes::Keyframes,
        timing::{Delay, Direction, Duration, Easing, IterationCount, Timing},
    };

    pub use crate::{TickPolicy, frame, subscription, subscription_with_policy};
}
