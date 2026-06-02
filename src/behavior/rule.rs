use crate::{
    behavior::{PropertyTransition, TransitionValueKind},
    property::{PropertySpec, PropertyValueKind},
    runtime::AnimationTargetId,
    timing::Timing,
};

/// Reusable animation behavior for value changes on one property.
///
/// A rule describes the property and timing independently from any concrete UI
/// target. Bind it to a target to create a [`PropertyTransition`] tracker.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BehaviorRule<K: PropertyValueKind> {
    property: PropertySpec<K>,
    timing: Timing,
}

impl<K: PropertyValueKind> BehaviorRule<K> {
    /// Creates a reusable rule with default timing.
    #[must_use]
    pub fn new(property: PropertySpec<K>) -> Self {
        Self {
            property,
            timing: Timing::default(),
        }
    }

    /// Replaces the timing used by transitions created from this rule.
    #[must_use]
    pub const fn with_timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }

    /// Returns the property animated by this rule.
    #[must_use]
    pub const fn property(&self) -> PropertySpec<K> {
        self.property
    }

    /// Returns the timing used by transitions created from this rule.
    #[must_use]
    pub const fn timing(&self) -> Timing {
        self.timing
    }

    /// Creates a target-bound value change tracker from this rule.
    #[must_use]
    pub fn bind(self, target: AnimationTargetId) -> PropertyTransition<K>
    where
        K: TransitionValueKind,
        K::Inner: Copy + PartialEq,
    {
        PropertyTransition::from_rule(target, &self)
    }
}
