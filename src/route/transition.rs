use crate::{Duration, RouteIncomingMotion, RouteTransition, Timeline};

/// Paired outgoing and incoming timelines for switching between two routes.
#[derive(Debug, Clone, PartialEq)]
pub struct RouteScreenTransition<R>
where
    R: Copy + Eq,
{
    from: R,
    to: R,
    outgoing: Timeline,
    incoming: Timeline,
}

impl<R> RouteScreenTransition<R>
where
    R: Copy + Eq,
{
    /// Creates a route screen transition with separate outgoing and incoming timelines.
    #[must_use]
    pub const fn new(from: R, to: R, outgoing: Timeline, incoming: Timeline) -> Self {
        Self {
            from,
            to,
            outgoing,
            incoming,
        }
    }

    /// Creates a route screen transition whose incoming screen fades and moves into place.
    #[must_use]
    pub fn with_incoming_motion(
        from: R,
        to: R,
        outgoing: Timeline,
        incoming: RouteIncomingMotion,
    ) -> Self {
        Self::new(from, to, outgoing, incoming.timeline())
    }

    /// Returns the route this transition starts from.
    #[must_use]
    pub const fn from(&self) -> R {
        self.from
    }

    /// Returns the route this transition targets.
    #[must_use]
    pub const fn to(&self) -> R {
        self.to
    }

    /// Returns the timeline registered for the outgoing screen.
    #[must_use]
    pub const fn outgoing(&self) -> &Timeline {
        &self.outgoing
    }

    /// Returns the timeline registered for the incoming screen.
    #[must_use]
    pub const fn incoming(&self) -> &Timeline {
        &self.incoming
    }

    /// Returns the finite screen transition duration, if both screen timelines are finite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        Some(
            self.outgoing
                .total_duration()?
                .max(self.incoming.total_duration()?),
        )
    }

    /// Returns whether the outgoing screen finishes before the incoming screen reaches its end.
    #[must_use]
    pub fn outgoing_shorter(&self) -> Option<bool> {
        Some(self.outgoing.total_duration()? <= self.incoming.total_duration()?)
    }

    pub(crate) fn route_transition(&self) -> RouteTransition<R> {
        RouteTransition::new(
            self.from,
            self.to,
            Timeline::hold(self.total_duration().unwrap_or(Duration::ZERO)),
        )
    }
}
