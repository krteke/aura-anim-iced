use crate::{
    AnimationHandle, AnimationRegistration, AnimationRuntime, AnimationTargetId, BehaviorRule,
    KeyframesBuilder, Timing, behavior::TransitionValueKind, property::PropertySpec,
    runtime::AnimationClock,
};

/// Tracks one visual property and starts a transition when its target value changes.
///
/// The first observed value becomes the stable target baseline and does not
/// start an animation. Later different values register a two-keyframe animation
/// from the previous visual result to the new target value.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTransition<K: TransitionValueKind>
where
    K::Inner: Copy + PartialEq,
{
    target: AnimationTargetId,
    property: PropertySpec<K>,
    timing: Timing,
    current: Option<K::Inner>,
    active: Option<AnimationHandle>,
}

impl<K> PropertyTransition<K>
where
    K: TransitionValueKind,
    K::Inner: Copy + PartialEq,
{
    /// Creates a property transition tracker with default timing.
    #[must_use]
    pub fn new(target: AnimationTargetId, property: PropertySpec<K>) -> Self {
        Self::from_rule(target, &BehaviorRule::new(property))
    }

    /// Creates a property transition tracker from a reusable behavior rule.
    #[must_use]
    pub const fn from_rule(target: AnimationTargetId, rule: &BehaviorRule<K>) -> Self {
        Self {
            target,
            property: rule.property(),
            timing: rule.timing(),
            current: None,
            active: None,
        }
    }

    /// Replaces the timing used for newly registered transitions.
    #[must_use]
    pub const fn with_timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }

    /// Returns the target that receives transition animations.
    #[must_use]
    pub const fn target(&self) -> AnimationTargetId {
        self.target
    }

    /// Returns the tracked property.
    #[must_use]
    pub const fn property(&self) -> PropertySpec<K> {
        self.property
    }

    /// Returns the timing used for newly registered transitions.
    #[must_use]
    pub const fn timing(&self) -> Timing {
        self.timing
    }

    /// Returns the last target value observed by this tracker.
    #[must_use]
    pub const fn current_value(&self) -> Option<K::Inner> {
        self.current
    }

    /// Returns the active runtime handle created by this tracker, if any.
    #[must_use]
    pub const fn active_handle(&self) -> Option<AnimationHandle> {
        self.active
    }

    /// Observes a new target value and registers an animation when it changed.
    ///
    /// Returns `None` when the value only seeded the baseline or did not change.
    /// If a previous transition is still running, the replacement starts from
    /// that transition's last sampled visual value.
    pub fn transition_to<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        value: K::Inner,
    ) -> Option<AnimationRegistration> {
        let Some(previous) = self.current else {
            self.current = Some(value);
            return None;
        };

        if previous == value {
            return None;
        }

        let from = self.current_visual_value(runtime).unwrap_or(previous);

        if let Some(active) = self.active.take() {
            runtime.cancel(self.target, active);
        }

        self.current = Some(value);

        let registration = runtime.register_keyframes(
            self.target,
            KeyframesBuilder::new()
                .with_timing(self.timing)
                .at(0.0, (self.property, from))
                .at(1.0, (self.property, value))
                .finish(),
        );

        self.active = Some(registration.handle());

        Some(registration)
    }

    fn current_visual_value<C: AnimationClock>(
        &self,
        runtime: &AnimationRuntime<C>,
    ) -> Option<K::Inner> {
        let active = self.active?;
        let snapshot = runtime.last_properties(self.target, active)?;
        let entry = snapshot.find_property(&self.property.raw())?;

        K::unwrap_transition_value(entry.value())
    }
}
