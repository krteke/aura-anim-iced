use crate::runtime::AnimationRuntime;

#[must_use]
pub fn should_subscribe(runtime: &AnimationRuntime) -> bool {
    !runtime.is_idle()
}
