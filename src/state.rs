//! State-driven animation helpers.

mod animator;
mod matcher;
mod progress;
mod transition;

pub use animator::StateAnimator;
pub use matcher::StateTransitionSet;
pub use progress::{ActiveStateTransition, StateTransitionProgress};
pub use transition::StateTransition;
