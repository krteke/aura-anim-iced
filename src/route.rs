//! Route transition helpers for screen-to-screen animation.
//!
//! Route animation builds on the state module and adds screen-specific helpers.
//! Use [`RouteAnimator`] for route state, [`RouteScreenTransition`] for paired
//! outgoing and incoming screen timelines, and [`RouteScreenTargets`] to keep
//! each screen's sampled properties separate.
//!
//! ```
//! use aura_anim_iced::{
//!     AnimationRuntime, AnimationTargetId, Duration, OPACITY, RouteAnimator,
//!     RouteIncomingMotion, RouteScreenTargets, RouteScreenTransition, Timeline,
//!     Track,
//! };
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! enum Route {
//!     Home,
//!     Settings,
//! }
//!
//! let mut runtime = AnimationRuntime::testing();
//! let route_target = AnimationTargetId::new();
//! let outgoing_target = AnimationTargetId::new();
//! let incoming_target = AnimationTargetId::new();
//! let mut animator = RouteAnimator::new(route_target, Route::Home);
//!
//! let transition = RouteScreenTransition::with_incoming_motion(
//!     Route::Home,
//!     Route::Settings,
//!     Timeline::track(
//!         Track::from(OPACITY, 1.0)
//!             .to(0.0)
//!             .duration(Duration::from_millis(80.0)),
//!     ),
//!     RouteIncomingMotion::new(
//!         iced::Vector::new(24.0, 0.0),
//!         Duration::from_millis(120.0),
//!     ),
//! );
//!
//! let registration = animator
//!     .transition_screens_with(
//!         &mut runtime,
//!         &transition,
//!         RouteScreenTargets::new(outgoing_target, incoming_target),
//!     )
//!     .expect("route screen transition");
//!
//! assert_eq!(animator.current(), Route::Settings);
//! assert_eq!(runtime.active_count(), 3);
//! assert_eq!(registration.outgoing().handle(), animator.active_screen_transition().unwrap().outgoing_handle());
//! ```

mod animator;
mod motion;
mod screen;
mod transition;

use crate::{
    ActiveStateTransition, StateTransition, StateTransitionRegistration, StateTransitionSet,
};

pub use animator::RouteAnimator;
pub use motion::RouteIncomingMotion;
pub use screen::{
    ActiveRouteScreenTransition, RouteScreenTargets, RouteScreenTransitionRegistration,
};
pub use transition::RouteScreenTransition;

/// Active route transition metadata tracked by a [`RouteAnimator`].
pub type ActiveRouteTransition<R> = ActiveStateTransition<R>;

/// Animation timeline for switching between two application routes.
pub type RouteTransition<R> = StateTransition<R>;

/// Route transition collection with optional fallback behavior.
pub type RouteTransitionSet<R> = StateTransitionSet<R>;

/// Output produced when a route transition is registered.
pub type RouteTransitionRegistration<R> = StateTransitionRegistration<R>;
