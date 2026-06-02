use std::hash::Hash;

use crate::{
    AnimationRuntime, PropertyTransition, RouteAnimator, StateAnimator, TransitionValueKind,
    runtime::AnimationClock,
};

/// Clears animation-owner metadata after runtime completion cleanup.
///
/// Runtime ticks remove finished animation entries automatically. Types that
/// cache active handles can implement this trait so product update code has one
/// cleanup path after a tick completes.
pub trait AnimationCompletionCleanup<C>
where
    C: AnimationClock,
{
    /// Clears stale active metadata against the current runtime state.
    ///
    /// Returns `true` when the owner changed.
    fn cleanup_completed(&mut self, runtime: &AnimationRuntime<C>) -> bool;
}

impl<C, K> AnimationCompletionCleanup<C> for PropertyTransition<K>
where
    C: AnimationClock,
    K: TransitionValueKind,
    K::Inner: Copy + PartialEq,
{
    fn cleanup_completed(&mut self, runtime: &AnimationRuntime<C>) -> bool {
        self.handle_completion(runtime)
    }
}

impl<C, S> AnimationCompletionCleanup<C> for StateAnimator<S>
where
    C: AnimationClock,
    S: Copy + Eq + Hash,
{
    fn cleanup_completed(&mut self, runtime: &AnimationRuntime<C>) -> bool {
        self.handle_completion(runtime)
    }
}

impl<C, R> AnimationCompletionCleanup<C> for RouteAnimator<R>
where
    C: AnimationClock,
    R: Copy + Eq + Hash,
{
    fn cleanup_completed(&mut self, runtime: &AnimationRuntime<C>) -> bool {
        self.handle_completion(runtime)
    }
}
