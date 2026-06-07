use std::time::{Duration, Instant};

use aura_anim_core::MotionRuntime;
use iced::Subscription;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TickPolicy {
    #[default]
    Frames,
    Interval(Duration),
}

impl TickPolicy {
    pub const fn frames() -> Self {
        Self::Frames
    }

    pub const fn interval(duration: Duration) -> Self {
        Self::Interval(duration)
    }

    pub fn fps(frames_per_second: u16) -> Self {
        Self::Interval(Duration::from_secs_f64(
            1.0 / f64::from(frames_per_second.max(1)),
        ))
    }
}

pub fn subscription(runtime: &MotionRuntime) -> Subscription<Instant> {
    subscription_with_policy(runtime, TickPolicy::Frames)
}

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

pub fn frame(runtime: &mut MotionRuntime, now: Instant) {
    runtime.tick_at(now);
}

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
