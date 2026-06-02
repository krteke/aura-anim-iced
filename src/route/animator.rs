use std::hash::Hash;

use crate::{
    route::{
        ActiveRouteScreenTransition, ActiveRouteTransition, RouteScreenTargets,
        RouteScreenTransitionRegistration, RouteTransition, RouteTransitionRegistration,
        RouteTransitionSet, transition::RouteScreenTransition,
    },
    runtime::{AnimationClock, AnimationHandle, AnimationRuntime, AnimationTargetId},
    state::StateAnimator,
};

/// Tracks the current application route and starts timelines for route changes.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteAnimator<R>
where
    R: Copy + Eq + Hash,
{
    inner: StateAnimator<R>,
    active_screen: Option<ActiveRouteScreenTransition<R>>,
}

impl<R> RouteAnimator<R>
where
    R: Copy + Eq + Hash,
{
    /// Creates a route animator for `target`.
    #[must_use]
    pub const fn new(target: AnimationTargetId, initial: R) -> Self {
        Self {
            inner: StateAnimator::new(target, initial),
            active_screen: None,
        }
    }

    /// Creates a route animator from the shared state animator implementation.
    #[must_use]
    pub const fn from_state_animator(inner: StateAnimator<R>) -> Self {
        Self {
            inner,
            active_screen: None,
        }
    }

    /// Returns the shared state animator used by this route animator.
    #[must_use]
    pub const fn as_state_animator(&self) -> &StateAnimator<R> {
        &self.inner
    }

    /// Converts this route animator into its shared state animator.
    #[must_use]
    pub fn into_state_animator(self) -> StateAnimator<R> {
        self.inner
    }

    /// Returns the target that receives route transition timelines.
    #[must_use]
    pub fn target(&self) -> AnimationTargetId {
        self.inner.target()
    }

    /// Returns the latest route observed by this animator.
    #[must_use]
    pub fn current(&self) -> R {
        self.inner.current()
    }

    /// Returns the active runtime handle created by this animator, if any.
    #[must_use]
    pub fn active_handle(&self) -> Option<AnimationHandle> {
        self.inner.active_handle()
    }

    /// Returns whether this animator currently owns a runtime animation handle.
    #[must_use]
    pub fn is_active<C: AnimationClock>(&self, runtime: &AnimationRuntime<C>) -> bool {
        self.inner.is_active(runtime)
    }

    /// Returns metadata for the active route transition, if any.
    #[must_use]
    pub fn active_transition(&self) -> Option<&ActiveRouteTransition<R>> {
        self.inner.active_transition()
    }

    /// Returns metadata for the active screen transition, if any.
    #[must_use]
    pub const fn active_screen_transition(&self) -> Option<&ActiveRouteScreenTransition<R>> {
        self.active_screen.as_ref()
    }

    /// Refreshes active transition metadata when its runtime handle is gone.
    ///
    /// Transition start methods refresh stale active metadata automatically.
    /// Call this when application code needs the cached active transition state
    /// to be accurate before starting another route transition.
    pub fn handle_completion<C: AnimationClock>(&mut self, runtime: &AnimationRuntime<C>) -> bool {
        let route_changed = self.inner.handle_completion(runtime);
        let screen_changed = self.invalidate_screen_if_stale(runtime);

        route_changed || screen_changed
    }

    /// Starts `transition` when it matches the animator's current route.
    ///
    /// Returns `None` when `transition` does not start from the current route,
    /// or when it would keep the route unchanged.
    pub fn transition_with<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        transition: &RouteTransition<R>,
    ) -> Option<RouteTransitionRegistration<R>> {
        self.invalidate_screen_if_stale(runtime);

        let registration = self.inner.transition_with(runtime, transition)?;
        let replaced = self.active_screen.take();

        cleanup_replaced_screen(runtime, replaced);

        Some(registration)
    }

    /// Finds and starts a transition from the current route to `to`.
    ///
    /// Uses the transition set fallback when no exact route-pair transition
    /// matches. Returns `None` when `to` is the current route or no transition
    /// behavior is available.
    pub fn transition_to<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        to: R,
        transitions: &RouteTransitionSet<R>,
    ) -> Option<RouteTransitionRegistration<R>> {
        self.invalidate_screen_if_stale(runtime);

        let registration = self.inner.transition_to(runtime, to, transitions)?;
        let replaced = self.active_screen.take();

        cleanup_replaced_screen(runtime, replaced);

        Some(registration)
    }

    /// Starts a route change with separate outgoing and incoming screen timelines.
    ///
    /// The route state transition is delegated to the shared state animator.
    /// Screen timelines are then registered on their own targets so the outgoing
    /// screen can animate before the incoming screen reaches its final state.
    pub fn transition_screens_with<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        transition: &RouteScreenTransition<R>,
        targets: RouteScreenTargets,
    ) -> Option<RouteScreenTransitionRegistration<R>> {
        self.invalidate_screen_if_stale(runtime);

        let route_transition = transition.route_transition();
        let route = self.inner.transition_with(runtime, &route_transition)?;
        let replaced = self.active_screen.take();

        cleanup_replaced_screen(runtime, replaced);

        let outgoing = runtime.register_timeline(targets.outgoing(), transition.outgoing().clone());
        let incoming = runtime.register_timeline(targets.incoming(), transition.incoming().clone());
        let active_route = *self.inner.active_transition()?;

        self.active_screen = Some(ActiveRouteScreenTransition::new(
            active_route,
            self.inner.target(),
            targets,
            outgoing.handle(),
            incoming.handle(),
        ));

        Some(RouteScreenTransitionRegistration::new(
            route, outgoing, incoming, replaced,
        ))
    }

    fn invalidate_screen_if_stale<C: AnimationClock>(
        &mut self,
        runtime: &AnimationRuntime<C>,
    ) -> bool {
        let Some(active) = self.active_screen else {
            return false;
        };

        if runtime.contains(active.route_target(), active.route().handle())
            || runtime.contains(active.outgoing_target(), active.outgoing_handle())
            || runtime.contains(active.incoming_target(), active.incoming_handle())
        {
            return false;
        }

        self.active_screen = None;
        true
    }
}

fn cleanup_replaced_screen<C, R>(
    runtime: &mut AnimationRuntime<C>,
    replaced: Option<ActiveRouteScreenTransition<R>>,
) where
    C: AnimationClock,
    R: Copy + Eq + Hash,
{
    if let Some(active) = replaced {
        runtime.cancel(active.route_target(), active.route().handle());
        runtime.cancel(active.outgoing_target(), active.outgoing_handle());
        runtime.cancel(active.incoming_target(), active.incoming_handle());
    }
}
