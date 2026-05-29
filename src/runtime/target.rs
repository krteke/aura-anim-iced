use uuid::Uuid;

use crate::PropertySnapshot;

/// Stable identity for one animated UI target.
///
/// A target usually maps to one widget, element, or view-model object in the
/// application. Runtime ticks keep snapshots scoped by this ID so unrelated
/// widgets never share a global property merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnimationTargetId(Uuid);

/// Property output grouped by animation target.
///
/// When multiple active sources target the same ID, their snapshots are merged
/// inside that target only. Composition never crosses target boundaries.
#[derive(Debug, Clone, PartialEq)]
pub struct TargetedPropertySnapshot {
    targets: Vec<(AnimationTargetId, PropertySnapshot)>,
}

impl AnimationTargetId {
    /// Creates a new random target ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AnimationTargetId {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetedPropertySnapshot {
    /// Creates an empty target-scoped snapshot.
    #[must_use]
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
        }
    }

    /// Returns whether no target produced properties.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }

    /// Returns the snapshot for `target_id`.
    #[must_use]
    pub fn get(&self, target_id: AnimationTargetId) -> Option<&PropertySnapshot> {
        self.targets
            .iter()
            .find(|(id, _)| *id == target_id)
            .map(|(_, snapshot)| snapshot)
    }

    pub(crate) fn merge(&mut self, target: AnimationTargetId, snapshot: PropertySnapshot) {
        if let Some(index) = self.targets.iter().position(|(id, _)| *id == target) {
            self.targets[index].1.merge(snapshot);
        } else {
            self.targets.push((target, snapshot));
        }
    }

    /// Returns all target snapshots in runtime merge order.
    #[must_use]
    pub fn targets(&self) -> &[(AnimationTargetId, PropertySnapshot)] {
        &self.targets
    }
}

impl Default for TargetedPropertySnapshot {
    fn default() -> Self {
        Self::new()
    }
}
