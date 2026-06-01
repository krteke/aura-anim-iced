//! Behavior helpers for property-driven animation.
//!
//! Use this module when a view value should animate because its target value
//! changed. A [`BehaviorRule`] stores reusable property and timing settings,
//! and binding it to an [`AnimationTargetId`](crate::AnimationTargetId) creates
//! a [`PropertyTransition`] that owns the current target value and active
//! runtime handle.
//!
//! ```
//! use aura_anim_iced::{
//!     AnimationRuntime, AnimationTargetId, BehaviorRule, Duration, PropertyValue,
//!     Timing, WIDTH,
//! };
//!
//! let mut runtime = AnimationRuntime::testing();
//! let target = AnimationTargetId::new();
//! let rule = BehaviorRule::new(WIDTH).with_timing(Timing::new(100.0));
//! let mut width = rule.bind(target);
//!
//! // The first value seeds the baseline and does not animate.
//! assert!(width.transition_to(&mut runtime, 120.0).is_none());
//!
//! let registration = width
//!     .transition_to(&mut runtime, 240.0)
//!     .expect("changed target starts an animation");
//! assert_eq!(width.active_handle(), Some(registration.handle()));
//!
//! runtime.clock_mut().set_now(Duration::from_millis(50.0));
//! let tick = runtime.tick();
//! let snapshot = tick.properties_for(target).expect("target output");
//! let entry = snapshot.find_property(&WIDTH.raw()).expect("width output");
//!
//! assert!(matches!(entry.value(), PropertyValue::Scalar(value) if (*value - 180.0).abs() < 0.001));
//! ```

mod progress;
mod registration;
mod rule;
mod transition;
mod value;

pub use progress::{ActivePropertyTransition, PropertyTransitionProgress};
pub use registration::PropertyTransitionRegistration;
pub use rule::BehaviorRule;
pub use transition::PropertyTransition;
pub use value::TransitionValueKind;
