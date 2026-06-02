use std::time::Instant;

use iced::Subscription;

use crate::{
    AnimationRegistration, AnimationRuntime, AnimationTargetId, AnimationTick,
    PropertyTransitionRegistration, RouteScreenTransitionRegistration, StateTransitionRegistration,
    runtime::{AnimationClock, SystemClock},
};

use super::{EffectSnapshot, subscription, tick_effect_snapshot_for};

/// Captures runtime registration output for the standard animation flow.
///
/// Existing value, behavior, state, and route helpers all produce registration
/// values that implement this trait. Future widget, theme, and spring helpers
/// can implement the same trait so application update code keeps one animation
/// output path.
pub trait AnimationFlowRegistration {
    /// Writes registration-time visual output into `tick`.
    fn capture_into(&self, tick: &mut AnimationTick);
}

/// Standard product integration flow for Iced applications.
///
/// The flow owns an [`AnimationRuntime`] and a reusable [`AnimationTick`]. Use
/// it from `update` to register animation work and process tick messages, from
/// `subscription` to keep Iced ticks active only while work is running, and
/// from `view` to read the latest sampled visual values.
#[derive(Debug, Clone)]
pub struct AnimationFlow<C = SystemClock> {
    runtime: AnimationRuntime<C>,
    output: AnimationTick,
}

impl AnimationFlow<SystemClock> {
    /// Creates a flow backed by a system-clock runtime.
    #[must_use]
    pub fn new() -> Self {
        Self::with_runtime(AnimationRuntime::new())
    }
}

impl Default for AnimationFlow<SystemClock> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> AnimationFlow<C> {
    /// Creates a flow around an existing runtime.
    #[must_use]
    pub fn with_runtime(runtime: AnimationRuntime<C>) -> Self {
        Self {
            runtime,
            output: AnimationTick::empty(),
        }
    }

    /// Returns the owned runtime.
    #[must_use]
    pub const fn runtime(&self) -> &AnimationRuntime<C> {
        &self.runtime
    }

    /// Returns mutable access to the owned runtime.
    pub const fn runtime_mut(&mut self) -> &mut AnimationRuntime<C> {
        &mut self.runtime
    }

    /// Returns the latest captured or sampled animation output.
    #[must_use]
    pub const fn output(&self) -> &AnimationTick {
        &self.output
    }

    /// Clears the latest output before collecting registration output in an
    /// application update branch.
    pub fn clear_output(&mut self) {
        self.output.clear();
    }

    /// Captures registration-time output into the flow output.
    pub fn capture(&mut self, registration: &impl AnimationFlowRegistration) {
        registration.capture_into(&mut self.output);
    }

    /// Returns whether the Iced app should keep the animation subscription.
    #[must_use]
    pub fn should_subscribe(&self) -> bool {
        self.runtime.should_subscribe()
    }

    /// Returns an Iced tick subscription while animation work is active.
    pub fn subscription<Message>(
        &self,
        map_tick: impl Fn(Instant) -> Message + Clone + Send + Sync + 'static,
    ) -> Subscription<Message>
    where
        Message: Send + 'static,
    {
        subscription(&self.runtime, map_tick)
    }

    /// Extracts view-friendly effects for a target from the latest output.
    #[must_use]
    pub fn effects_for(&self, target: AnimationTargetId) -> EffectSnapshot {
        tick_effect_snapshot_for(&self.output, target)
    }
}

impl<C> AnimationFlow<C>
where
    C: AnimationClock,
{
    /// Advances the runtime and stores the sampled output.
    pub fn tick(&mut self) -> &AnimationTick {
        self.runtime.tick_into(&mut self.output);

        &self.output
    }

    /// Routes an Iced tick message into the flow.
    pub fn update_tick(&mut self, _tick: Instant) -> &AnimationTick {
        self.tick()
    }
}

impl AnimationFlowRegistration for AnimationRegistration {
    fn capture_into(&self, tick: &mut AnimationTick) {
        tick.capture_registration(self);
    }
}

impl AnimationFlowRegistration for PropertyTransitionRegistration {
    fn capture_into(&self, tick: &mut AnimationTick) {
        self.registration().capture_into(tick);
    }
}

impl<S> AnimationFlowRegistration for StateTransitionRegistration<S>
where
    S: Copy + Eq,
{
    fn capture_into(&self, tick: &mut AnimationTick) {
        self.registration().capture_into(tick);
    }
}

impl<R> AnimationFlowRegistration for RouteScreenTransitionRegistration<R>
where
    R: Copy + Eq,
{
    fn capture_into(&self, tick: &mut AnimationTick) {
        self.route().capture_into(tick);
        self.outgoing().capture_into(tick);
        self.incoming().capture_into(tick);
    }
}
