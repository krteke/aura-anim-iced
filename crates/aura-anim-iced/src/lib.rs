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
        Animatable, Animation, AnimationCommand, AnimationExt, AnimationState, BoxAnimation, Hold,
        Interpolate, InterpolationProgress, Motion, MotionBinding, MotionBindingError,
        MotionBindingState, MotionRuntime, Parallel, Presence, RetainPolicy, Sequence, Spring,
        SpringConfig, Timeline, TransitionContext, Tween, TweenState,
        keyframes::Keyframes,
        timing::{Delay, Direction, Duration, Easing, IterationCount, Timing},
    };

    pub use crate::{TickPolicy, frame, subscription, subscription_with_policy};
}

#[cfg(test)]
mod tests {
    use super::{TickPolicy, frame, subscription, subscription_with_policy};
    use aura_anim_core::{MotionRuntime, timing::Timing};
    use float_cmp::assert_approx_eq;
    use std::time::{Duration, Instant};

    #[test]
    fn default_policy_uses_window_frames() {
        assert_eq!(TickPolicy::default(), TickPolicy::Frames);
        assert_eq!(TickPolicy::frames(), TickPolicy::Frames);
    }

    #[test]
    fn interval_policy_preserves_duration() {
        let duration = Duration::from_millis(20);

        assert_eq!(
            TickPolicy::interval(duration),
            TickPolicy::Interval(duration)
        );
    }

    #[test]
    fn fps_policy_converts_rate_to_interval() {
        assert_eq!(
            TickPolicy::fps(50),
            TickPolicy::Interval(Duration::from_millis(20))
        );
        assert_eq!(
            TickPolicy::fps(0),
            TickPolicy::Interval(Duration::from_secs(1))
        );
    }

    #[test]
    fn subscriptions_can_be_built_for_idle_runtime() {
        let runtime = MotionRuntime::new();

        let frames = subscription(&runtime);
        let interval =
            subscription_with_policy(&runtime, TickPolicy::interval(Duration::from_millis(10)));

        let _ = (frames, interval);
    }

    #[test]
    fn frame_advances_runtime_using_instants() {
        let mut runtime = MotionRuntime::new();
        let motion = runtime.motion_with(0.0_f32, Timing::new(100.0));
        assert!(motion.transition_to(10.0, &mut runtime));
        let start = Instant::now();

        frame(&mut runtime, start);
        frame(&mut runtime, start + Duration::from_millis(50));

        assert_approx_eq!(f32, motion.value(&runtime), 5.0, epsilon = 0.001);
    }
}
