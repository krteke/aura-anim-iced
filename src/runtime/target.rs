use rustc_hash::FxHashMap;
use smallvec::SmallVec;
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
    targets: FxHashMap<AnimationTargetId, PropertySnapshot>,
    order: SmallVec<[AnimationTargetId; 16]>,
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
            targets: FxHashMap::default(),
            order: SmallVec::new(),
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
        self.targets.get(&target_id)
    }

    pub(crate) fn merge(&mut self, target: AnimationTargetId, snapshot: PropertySnapshot) {
        if let Some(entry) = self.targets.get_mut(&target) {
            entry.merge(snapshot);
        } else {
            self.order.push(target);
        }
    }

    /// Returns all target snapshots in runtime merge order.
    pub fn targets(&self) -> impl Iterator<Item = (AnimationTargetId, &PropertySnapshot)> + '_ {
        self.order.iter().map(|id| (*id, &self.targets[id]))
    }
}

impl Default for TargetedPropertySnapshot {
    fn default() -> Self {
        Self::new()
    }
}
