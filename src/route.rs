//! Route transition helpers for screen-to-screen animation.

mod animator;

use crate::{
    ActiveStateTransition, StateTransition, StateTransitionRegistration, StateTransitionSet,
};

pub use animator::RouteAnimator;

/// Active route transition metadata tracked by a [`RouteAnimator`].
pub type ActiveRouteTransition<R> = ActiveStateTransition<R>;

/// Animation timeline for switching between two application routes.
pub type RouteTransition<R> = StateTransition<R>;

/// Route transition collection with optional fallback behavior.
pub type RouteTransitionSet<R> = StateTransitionSet<R>;

/// Output produced when a route transition is registered.
pub type RouteTransitionRegistration<R> = StateTransitionRegistration<R>;
