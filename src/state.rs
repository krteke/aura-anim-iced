//! State-driven animation helpers.
//!
//! Use this module when application states should map to reusable animation
//! timelines. A [`StateTransitionSet`] matches `(from, to)` pairs and
//! [`StateAnimator`] registers the matching timeline against one runtime target.
//!
//! ```
//! use aura_anim_iced::{
//!     AnimationRuntime, AnimationTargetId, Duration, OPACITY, PropertyValue,
//!     StateAnimator, StateTransition, StateTransitionSet, Timeline, Track,
//! };
//!
//! #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! enum Panel {
//!     Closed,
//!     Open,
//! }
//!
//! fn fade(from: f32, to: f32) -> Timeline {
//!     Timeline::track(
//!         Track::from(OPACITY, from)
//!             .to(to)
//!             .duration(Duration::from_millis(100.0)),
//!     )
//! }
//!
//! let mut runtime = AnimationRuntime::testing();
//! let target = AnimationTargetId::new();
//! let transitions = StateTransitionSet::from_transitions([
//!     StateTransition::new(Panel::Closed, Panel::Open, fade(0.0, 1.0)),
//! ]);
//! let mut animator = StateAnimator::new(target, Panel::Closed);
//!
//! let registration = animator
//!     .transition_to(&mut runtime, Panel::Open, &transitions)
//!     .expect("closed to open transition");
//! assert_eq!(animator.active_handle(), Some(registration.handle()));
//!
//! runtime.clock_mut().set_now(Duration::from_millis(50.0));
//! let tick = runtime.tick();
//! let entry = tick
//!     .properties_for(target)
//!     .unwrap()
//!     .find_property(&OPACITY.raw())
//!     .unwrap();
//! assert!(matches!(entry.value(), PropertyValue::Scalar(value) if (*value - 0.5).abs() < 0.001));
//! ```

mod animator;
mod matcher;
mod progress;
mod registration;
mod transition;

pub use animator::StateAnimator;
pub use matcher::StateTransitionSet;
pub use progress::{ActiveStateTransition, StateTransitionProgress};
pub use registration::StateTransitionRegistration;
pub use transition::StateTransition;
