//! Iced integration helpers.

use iced::Subscription;
use std::time::{Duration as StdDuration, Instant};

use crate::runtime::AnimationRuntime;

/// Returns whether the runtime should keep an Iced tick subscription active.
#[must_use]
pub fn should_subscribe<C>(runtime: &AnimationRuntime<C>) -> bool {
    runtime.should_subscribe()
}

/// Returns an Iced tick subscription while the runtime has playing animations.
pub fn subscription<Message, C>(
    runtime: &AnimationRuntime<C>,
    map_tick: impl Fn(Instant) -> Message + Clone + Send + Sync + 'static,
) -> Subscription<Message>
where
    Message: Send + 'static,
{
    if !runtime.should_subscribe() {
        return Subscription::none();
    }

    iced::time::every(std_tick_interval(runtime)).map(map_tick)
}

fn std_tick_interval<C>(runtime: &AnimationRuntime<C>) -> StdDuration {
    let millis = runtime.motion_policy().tick_interval().as_millis();

    StdDuration::from_secs_f64((millis / 1000.0).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::{should_subscribe, subscription};
    use crate::{
        keyframes::Keyframes,
        runtime::AnimationRuntime,
        timing::{Duration, Timing},
    };

    #[test]
    fn subscription_gate_tracks_runtime_activity() {
        let mut runtime = AnimationRuntime::testing();

        assert!(!should_subscribe(&runtime));

        runtime.register_keyframes(
            Keyframes::new()
                .with_timing(Timing::new(100.0))
                .opacity(0.0, 0.0)
                .opacity(1.0, 1.0),
        );

        assert!(should_subscribe(&runtime));

        runtime.clock_mut().set_now(Duration::from_millis(100.0));
        runtime.tick();

        assert!(!should_subscribe(&runtime));
    }

    #[test]
    fn subscription_helper_compiles_for_idle_and_active_runtime() {
        let mut runtime = AnimationRuntime::testing();

        let _idle = subscription(&runtime, |_| ());

        runtime.register_keyframes(
            Keyframes::new()
                .with_timing(Timing::new(100.0))
                .opacity(0.0, 0.0)
                .opacity(1.0, 1.0),
        );

        let _active = subscription(&runtime, |_| ());
    }
}
