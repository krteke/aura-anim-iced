use crate::{
    ActiveRouteTransition, AnimationHandle, AnimationRegistration, AnimationTargetId,
    RouteTransitionRegistration,
};

/// Active screen-to-screen route transition metadata tracked by a [`RouteAnimator`](crate::RouteAnimator).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveRouteScreenTransition<R>
where
    R: Copy + Eq,
{
    route: ActiveRouteTransition<R>,
    route_target: AnimationTargetId,
    outgoing_target: AnimationTargetId,
    outgoing_handle: AnimationHandle,
    incoming_target: AnimationTargetId,
    incoming_handle: AnimationHandle,
}

impl<R> ActiveRouteScreenTransition<R>
where
    R: Copy + Eq,
{
    pub(crate) const fn new(
        route: ActiveRouteTransition<R>,
        route_target: AnimationTargetId,
        targets: RouteScreenTargets,
        outgoing_handle: AnimationHandle,
        incoming_handle: AnimationHandle,
    ) -> Self {
        Self {
            route,
            route_target,
            outgoing_target: targets.outgoing(),
            outgoing_handle,
            incoming_target: targets.incoming(),
            incoming_handle,
        }
    }

    /// Returns the active route state transition metadata.
    #[must_use]
    pub const fn route(&self) -> &ActiveRouteTransition<R> {
        &self.route
    }

    /// Returns the target used by the route state transition.
    #[must_use]
    pub const fn route_target(&self) -> AnimationTargetId {
        self.route_target
    }

    /// Returns the target used by the screen that is leaving.
    #[must_use]
    pub const fn outgoing_target(&self) -> AnimationTargetId {
        self.outgoing_target
    }

    /// Returns the runtime handle for the outgoing screen animation.
    #[must_use]
    pub const fn outgoing_handle(&self) -> AnimationHandle {
        self.outgoing_handle
    }

    /// Returns the target used by the screen that is entering.
    #[must_use]
    pub const fn incoming_target(&self) -> AnimationTargetId {
        self.incoming_target
    }

    /// Returns the runtime handle for the incoming screen animation.
    #[must_use]
    pub const fn incoming_handle(&self) -> AnimationHandle {
        self.incoming_handle
    }
}

/// Runtime targets used by a screen-to-screen route transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteScreenTargets {
    outgoing: AnimationTargetId,
    incoming: AnimationTargetId,
}

impl RouteScreenTargets {
    /// Creates target bindings for outgoing and incoming screens.
    #[must_use]
    pub const fn new(outgoing: AnimationTargetId, incoming: AnimationTargetId) -> Self {
        Self { outgoing, incoming }
    }

    /// Returns the target used by the screen that is leaving.
    #[must_use]
    pub const fn outgoing(&self) -> AnimationTargetId {
        self.outgoing
    }

    /// Returns the target used by the screen that is entering.
    #[must_use]
    pub const fn incoming(&self) -> AnimationTargetId {
        self.incoming
    }
}

/// Runtime registrations produced by a screen-to-screen route transition.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteScreenTransitionRegistration<R>
where
    R: Copy + Eq,
{
    route: RouteTransitionRegistration<R>,
    outgoing: AnimationRegistration,
    incoming: AnimationRegistration,
    replaced: Option<ActiveRouteScreenTransition<R>>,
}

impl<R> RouteScreenTransitionRegistration<R>
where
    R: Copy + Eq,
{
    pub(crate) const fn new(
        route: RouteTransitionRegistration<R>,
        outgoing: AnimationRegistration,
        incoming: AnimationRegistration,
        replaced: Option<ActiveRouteScreenTransition<R>>,
    ) -> Self {
        Self {
            route,
            outgoing,
            incoming,
            replaced,
        }
    }

    /// Returns the route state transition registration.
    #[must_use]
    pub const fn route(&self) -> &RouteTransitionRegistration<R> {
        &self.route
    }

    /// Returns the outgoing screen animation registration.
    #[must_use]
    pub const fn outgoing(&self) -> &AnimationRegistration {
        &self.outgoing
    }

    /// Returns the incoming screen animation registration.
    #[must_use]
    pub const fn incoming(&self) -> &AnimationRegistration {
        &self.incoming
    }

    /// Returns the active screen transition replaced by this registration.
    #[must_use]
    pub const fn replaced(&self) -> Option<&ActiveRouteScreenTransition<R>> {
        self.replaced.as_ref()
    }
}
