//! Behavior helpers for property-driven animation.

mod registration;
mod rule;
mod transition;
mod value;

pub use registration::PropertyTransitionRegistration;
pub use rule::BehaviorRule;
pub use transition::PropertyTransition;
pub use value::TransitionValueKind;
