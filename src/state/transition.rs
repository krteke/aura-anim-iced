use crate::Timeline;

/// Animation timeline for moving between two application states.
#[derive(Debug, Clone, PartialEq)]
pub struct StateTransition<S>
where
    S: Copy + Eq,
{
    from: S,
    to: S,
    timeline: Timeline,
}

impl<S> StateTransition<S>
where
    S: Copy + Eq,
{
    /// Creates a state transition backed by a timeline.
    #[must_use]
    pub const fn new(from: S, to: S, timeline: Timeline) -> Self {
        Self { from, to, timeline }
    }

    /// Returns the state this transition starts from.
    #[must_use]
    pub const fn from(&self) -> S {
        self.from
    }

    /// Returns the state this transition ends at.
    #[must_use]
    pub const fn to(&self) -> S {
        self.to
    }

    /// Returns the timeline registered when this transition starts.
    #[must_use]
    pub const fn timeline(&self) -> &Timeline {
        &self.timeline
    }
}
