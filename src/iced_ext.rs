//! Iced integration helpers.

use crate::runtime::AnimationRuntime;

/// Returns whether the runtime should keep an Iced tick subscription active.
#[must_use]
pub fn should_subscribe<C>(runtime: &AnimationRuntime<C>) -> bool {
    runtime.should_subscribe()
}
