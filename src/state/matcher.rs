use crate::StateTransition;

/// Reusable collection that matches state pairs to transition timelines.
#[derive(Debug, Clone, PartialEq)]
pub struct StateTransitionSet<S>
where
    S: Copy + Eq,
{
    transitions: Vec<StateTransition<S>>,
}

impl<S> StateTransitionSet<S>
where
    S: Copy + Eq,
{
    /// Creates an empty transition set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            transitions: Vec::new(),
        }
    }

    /// Creates a transition set from an iterator of transitions.
    #[must_use]
    pub fn from_transitions(transitions: impl IntoIterator<Item = StateTransition<S>>) -> Self {
        Self {
            transitions: transitions.into_iter().collect(),
        }
    }

    /// Adds a transition to the set.
    pub fn push(&mut self, transition: StateTransition<S>) {
        self.transitions.push(transition);
    }

    /// Returns all transitions in insertion order.
    #[must_use]
    pub fn transitions(&self) -> &[StateTransition<S>] {
        &self.transitions
    }

    /// Returns the first transition matching `from` and `to`.
    #[must_use]
    pub fn find(&self, from: S, to: S) -> Option<&StateTransition<S>> {
        self.transitions
            .iter()
            .find(|transition| transition.from() == from && transition.to() == to)
    }
}

impl<S> Default for StateTransitionSet<S>
where
    S: Copy + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> From<Vec<StateTransition<S>>> for StateTransitionSet<S>
where
    S: Copy + Eq,
{
    fn from(value: Vec<StateTransition<S>>) -> Self {
        Self::from_transitions(value)
    }
}
