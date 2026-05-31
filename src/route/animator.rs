use crate::{
    ActiveRouteTransition, AnimationHandle, AnimationRuntime, AnimationTargetId,
    RouteScreenTargets, RouteScreenTransition, RouteScreenTransitionRegistration, RouteTransition,
    RouteTransitionRegistration, RouteTransitionSet, StateAnimator, runtime::AnimationClock,
};

/// Tracks the current application route and starts timelines for route changes.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteAnimator<R>
where
    R: Copy + Eq,
{
    inner: StateAnimator<R>,
}

impl<R> RouteAnimator<R>
where
    R: Copy + Eq,
{
    /// Creates a route animator for `target`.
    #[must_use]
    pub const fn new(target: AnimationTargetId, initial: R) -> Self {
        Self {
            inner: StateAnimator::new(target, initial),
        }
    }

    /// Creates a route animator from the shared state animator implementation.
    #[must_use]
    pub const fn from_state_animator(inner: StateAnimator<R>) -> Self {
        Self { inner }
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

    /// Refreshes active transition metadata when its runtime handle is gone.
    ///
    /// Transition start methods refresh stale active metadata automatically.
    /// Call this when application code needs the cached active transition state
    /// to be accurate before starting another route transition.
    pub fn handle_completion<C: AnimationClock>(&mut self, runtime: &AnimationRuntime<C>) -> bool {
        self.inner.handle_completion(runtime)
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
        self.inner.transition_with(runtime, transition)
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
        self.inner.transition_to(runtime, to, transitions)
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
        let route_transition = transition.route_transition();
        let route = self.transition_with(runtime, &route_transition)?;
        let outgoing = runtime.register_timeline(targets.outgoing(), transition.outgoing().clone());
        let incoming = runtime.register_timeline(targets.incoming(), transition.incoming().clone());

        Some(RouteScreenTransitionRegistration::new(
            route, outgoing, incoming,
        ))
    }
}
