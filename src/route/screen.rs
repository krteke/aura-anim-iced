use crate::{AnimationRegistration, AnimationTargetId, RouteTransitionRegistration};

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
}

impl<R> RouteScreenTransitionRegistration<R>
where
    R: Copy + Eq,
{
    pub(crate) const fn new(
        route: RouteTransitionRegistration<R>,
        outgoing: AnimationRegistration,
        incoming: AnimationRegistration,
    ) -> Self {
        Self {
            route,
            outgoing,
            incoming,
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
}
