use std::{hash::Hash, sync::Arc};

use rustc_hash::FxHashMap;

use crate::{state::StateTransition, timeline::Timeline};

/// Reusable collection that matches state pairs to transition timelines.
#[derive(Debug, Clone, PartialEq)]
pub struct StateTransitionSet<S>
where
    S: Copy + Eq + Hash,
{
    transitions: Vec<StateTransition<S>>,
    index: FxHashMap<(S, S), usize>,
    fallback: Option<Arc<Timeline>>,
}

impl<S> StateTransitionSet<S>
where
    S: Copy + Eq + Hash,
{
    /// Creates an empty transition set.
    #[must_use]
    pub fn new() -> Self {
        Self {
            transitions: Vec::new(),
            index: FxHashMap::default(),
            fallback: None,
        }
    }

    /// Creates a transition set from an iterator of transitions.
    #[must_use]
    pub fn from_transitions(transitions: impl IntoIterator<Item = StateTransition<S>>) -> Self {
        let mut set = Self::new();

        for transition in transitions {
            set.push(transition);
        }

        set
    }

    /// Sets the fallback timeline used when no state-pair transition matches.
    #[must_use]
    pub fn with_fallback(mut self, fallback: Timeline) -> Self {
        self.fallback = Some(Arc::new(fallback));
        self
    }

    /// Adds a transition to the set.
    pub fn push(&mut self, transition: StateTransition<S>) {
        let key = (transition.from(), transition.to());
        let index = self.transitions.len();

        self.transitions.push(transition);
        self.index.entry(key).or_insert(index);
    }

    /// Returns all transitions in insertion order.
    #[must_use]
    pub fn transitions(&self) -> &[StateTransition<S>] {
        &self.transitions
    }

    /// Returns the fallback timeline, if one is configured.
    #[must_use]
    pub fn fallback(&self) -> Option<&Timeline> {
        self.fallback.as_deref()
    }

    pub(crate) fn fallback_arc(&self) -> Option<Arc<Timeline>> {
        self.fallback.as_ref().map(Arc::clone)
    }

    /// Returns the first transition matching `from` and `to`.
    #[must_use]
    pub fn find(&self, from: S, to: S) -> Option<&StateTransition<S>> {
        self.index
            .get(&(from, to))
            .and_then(|index| self.transitions.get(*index))
    }
}

impl<S> Default for StateTransitionSet<S>
where
    S: Copy + Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> From<Vec<StateTransition<S>>> for StateTransitionSet<S>
where
    S: Copy + Eq + Hash,
{
    fn from(value: Vec<StateTransition<S>>) -> Self {
        Self::from_transitions(value)
    }
}
