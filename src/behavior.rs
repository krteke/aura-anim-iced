//! Behavior helpers for property-driven animation.

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
