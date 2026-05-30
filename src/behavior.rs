//! Behavior helpers for property-driven animation.

use crate::{
    AnimationRegistration, AnimationRuntime, AnimationTargetId, KeyframesBuilder, Timing,
    property::{PropertySpec, PropertyValueKind},
    runtime::AnimationClock,
};

/// Tracks one visual property and starts a transition when its target value changes.
///
/// The first observed value becomes the stable baseline and does not start an
/// animation. Later different values register a two-keyframe animation from the
/// previous stable value to the new target value.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTransition<K: PropertyValueKind>
where
    K::Inner: Copy + PartialEq,
{
    target: AnimationTargetId,
    property: PropertySpec<K>,
    timing: Timing,
    current: Option<K::Inner>,
}

impl<K> PropertyTransition<K>
where
    K: PropertyValueKind,
    K::Inner: Copy + PartialEq,
{
    /// Creates a property transition tracker with default timing.
    #[must_use]
    pub fn new(target: AnimationTargetId, property: PropertySpec<K>) -> Self {
        Self {
            target,
            property,
            timing: Timing::default(),
            current: None,
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

    /// Returns the last stable target value observed by this tracker.
    #[must_use]
    pub const fn current_value(&self) -> Option<K::Inner> {
        self.current
    }

    /// Observes a new target value and registers an animation when it changed.
    ///
    /// Returns `None` when the value only seeded the baseline or did not change.
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

        self.current = Some(value);

        Some(
            runtime.register_keyframes(
                self.target,
                KeyframesBuilder::new()
                    .with_timing(self.timing)
                    .at(0.0, (self.property, previous))
                    .at(1.0, (self.property, value))
                    .finish(),
            ),
        )
    }
}
