//! State-driven animation helpers.

mod animator;
mod matcher;
mod transition;

pub use animator::StateAnimator;
pub use matcher::StateTransitionSet;
pub use transition::StateTransition;
