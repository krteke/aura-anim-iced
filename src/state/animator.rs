use crate::{
    AnimationHandle, AnimationRegistration, AnimationRuntime, AnimationTargetId, StateTransition,
    StateTransitionSet, runtime::AnimationClock,
};

/// Tracks an application state and starts timelines for explicit state changes.
#[derive(Debug, Clone, PartialEq)]
pub struct StateAnimator<S>
where
    S: Copy + Eq,
{
    target: AnimationTargetId,
    current: S,
    active: Option<AnimationHandle>,
}

impl<S> StateAnimator<S>
where
    S: Copy + Eq,
{
    /// Creates a state animator for `target`.
    #[must_use]
    pub const fn new(target: AnimationTargetId, initial: S) -> Self {
        Self {
            target,
            current: initial,
            active: None,
        }
    }

    /// Returns the target that receives transition timelines.
    #[must_use]
    pub const fn target(&self) -> AnimationTargetId {
        self.target
    }

    /// Returns the latest application state observed by this animator.
    #[must_use]
    pub const fn current(&self) -> S {
        self.current
    }

    /// Returns the active runtime handle created by this animator, if any.
    #[must_use]
    pub const fn active_handle(&self) -> Option<AnimationHandle> {
        self.active
    }

    /// Returns whether this animator currently owns a runtime animation handle.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Starts `transition` when it matches the animator's current state.
    ///
    /// Returns `None` when `transition` does not start from the current state,
    /// or when it would keep the state unchanged.
    pub fn transition_with<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        transition: &StateTransition<S>,
    ) -> Option<AnimationRegistration> {
        if self.current != transition.from() || transition.from() == transition.to() {
            return None;
        }

        if let Some(active) = self.active.take() {
            runtime.cancel(self.target, active);
        }

        self.current = transition.to();

        let registration = runtime.register_timeline(self.target, transition.timeline().clone());

        self.active = Some(registration.handle());

        Some(registration)
    }

    /// Finds and starts a transition from the current state to `to`.
    ///
    /// Returns `None` when no matching transition exists, or when `to` is the
    /// current state.
    pub fn transition_to<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        to: S,
        transitions: &StateTransitionSet<S>,
    ) -> Option<AnimationRegistration> {
        if self.current == to {
            return None;
        }

        let transition = transitions.find(self.current, to)?;

        self.transition_with(runtime, transition)
    }
}
