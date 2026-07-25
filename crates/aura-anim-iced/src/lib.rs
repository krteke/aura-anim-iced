//! Iced subscriptions and frame handling for [`aura_anim_core::runtime::MotionRuntime`].

use std::time::{Duration, Instant};

use aura_anim_core::runtime::MotionRuntime;
use iced::{Event, Subscription, event, window};

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
    ///
    /// A rate of zero is treated as one frame per second.
    #[must_use]
    pub fn fps(frames_per_second: u16) -> Self {
        Self::Interval(Duration::from_secs_f64(
            1.0 / f64::from(frames_per_second.max(1)),
        ))
    }
}

/// A timestamped animation tick associated with an Iced window.
///
/// Window-specific subscriptions yield this value so applications can update the
/// matching runtime with [`Subscribe::frame`] and retain the window identity.
pub struct WindowFrame {
    /// The window that requested the redraw or owns the interval tick.
    pub window: iced::window::Id,
    /// The instant at which the tick occurred.
    pub at: Instant,
}

/// Extends [`MotionRuntime`] with Iced frame handling and subscriptions.
///
/// Import this trait to call its methods on a runtime. Rebuild the returned
/// subscription from an application's subscription function so it is active
/// only while the runtime has animations to advance.
pub trait Subscribe {
    /// Advances the runtime using an Iced frame timestamp.
    fn frame(&mut self, now: Instant);

    /// Creates a frame-driven subscription while the runtime has active animations.
    ///
    /// The subscription yields Iced frame timestamps without window identity.
    fn subscription(&self) -> Subscription<Instant>;

    /// Creates a subscription using `tick_policy` while the runtime has active animations.
    ///
    /// The subscription yields timestamps without window identity.
    fn subscription_with_policy(&self, tick_policy: TickPolicy) -> Subscription<Instant>;

    /// Creates a frame-driven subscription for one window while animations are active.
    ///
    /// The subscription yields only redraw ticks for `window`.
    fn subscription_for(&self, window: iced::window::Id) -> Subscription<WindowFrame>;

    /// Creates a window-specific subscription using `tick_policy` while animations are active.
    ///
    /// Frame-driven ticks are filtered to `window`; interval ticks are labelled
    /// with `window` in the returned [`WindowFrame`].
    fn subscription_with_policy_for(
        &self,
        tick_policy: TickPolicy,
        window: iced::window::Id,
    ) -> Subscription<WindowFrame>;
}

impl Subscribe for MotionRuntime {
    fn frame(&mut self, now: Instant) {
        self.tick_at(now);
    }

    /// Creates a frame-driven subscription while the runtime has active animations.
    fn subscription(&self) -> Subscription<Instant> {
        self.subscription_with_policy(TickPolicy::Frames)
    }

    /// Creates a subscription using `tick_policy` while the runtime has active animations.
    ///
    /// # Examples
    ///
    /// ```
    /// use aura_anim_core::runtime::MotionRuntime;
    /// use aura_anim_iced::{Subscribe, TickPolicy};
    ///
    /// let runtime = MotionRuntime::new();
    /// let subscription = runtime.subscription_with_policy(TickPolicy::fps(60));
    /// # let _ = subscription;
    /// ```
    fn subscription_with_policy(&self, tick_policy: TickPolicy) -> Subscription<Instant> {
        if !self.has_active() {
            return Subscription::none();
        }

        match tick_policy {
            TickPolicy::Frames => iced::window::frames(),
            TickPolicy::Interval(duration) => {
                iced::time::every(duration.max(Duration::from_millis(1)))
            }
        }
    }

    fn subscription_for(&self, window: iced::window::Id) -> Subscription<WindowFrame> {
        self.subscription_with_policy_for(TickPolicy::Frames, window)
    }

    fn subscription_with_policy_for(
        &self,
        tick_policy: TickPolicy,
        window: iced::window::Id,
    ) -> Subscription<WindowFrame> {
        if !self.has_active() {
            return Subscription::none();
        }

        match tick_policy {
            TickPolicy::Frames => event::listen_raw(|event, _, window| match event {
                Event::Window(window::Event::RedrawRequested(at)) => {
                    Some(WindowFrame { window, at })
                }
                _ => None,
            })
            .with(window)
            .filter_map(|(window, frame)| {
                if window == frame.window {
                    Some(frame)
                } else {
                    None
                }
            }),
            TickPolicy::Interval(duration) => {
                iced::time::every(duration.max(Duration::from_millis(1)))
                    .with(window)
                    .map(|(window, at)| WindowFrame { window, at })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Subscribe;

    use super::TickPolicy;
    use aura_anim_core::{runtime::MotionRuntime, timing::Timing};
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

        let frames = runtime.subscription();
        let interval =
            runtime.subscription_with_policy(TickPolicy::Interval(Duration::from_millis(10)));

        let _ = (frames, interval);
    }

    #[test]
    fn frame_advances_runtime_using_instants() {
        let mut runtime = MotionRuntime::new();
        let motion = runtime.motion_with(0.0_f32, Timing::new(100.0));
        assert!(motion.transition_to(10.0, &mut runtime).is_ok());
        let start = Instant::now();

        runtime.frame(start);
        runtime.frame(start + Duration::from_millis(50));

        assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 5.0, epsilon = 0.001);
    }
}
