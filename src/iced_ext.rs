//! Iced integration helpers.

mod effect;

use iced::Subscription;
use std::time::{Duration as StdDuration, Instant};

use crate::runtime::{AnimationClock, AnimationRuntime, AnimationTick};

#[cfg(test)]
#[cfg(feature = "testing")]
mod tests;

pub use effect::{EffectSnapshot, effect_snapshot, tick_effect_snapshot};

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

/// Routes an Iced tick message into the runtime and returns the sampled output.
pub fn update_tick<C>(runtime: &mut AnimationRuntime<C>, _tick: Instant) -> AnimationTick
where
    C: AnimationClock,
{
    runtime.tick()
}

fn std_tick_interval<C>(runtime: &AnimationRuntime<C>) -> StdDuration {
    let millis = runtime.motion_policy().tick_interval().as_millis();

    StdDuration::from_secs_f64((millis / 1000.0).max(0.0))
}
