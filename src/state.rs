//! State-driven animation helpers.

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
