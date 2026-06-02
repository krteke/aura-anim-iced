use rustc_hash::FxHashMap;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::property::PropertySnapshot;

static NEXT_TARGET_ID: AtomicU64 = AtomicU64::new(AnimationTargetId::FIRST_ID);

/// Stable identity for one animated UI target.
///
/// A target usually maps to one widget, element, or view-model object in the
/// application. Runtime ticks keep snapshots scoped by this ID so unrelated
/// widgets never share a global property merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AnimationTargetId(u64);

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
    const FIRST_ID: u64 = 1;

    /// Creates a new process-local target ID.
    #[must_use]
    pub fn new() -> Self {
        Self(NEXT_TARGET_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the numeric target ID.
    #[must_use]
    pub const fn id(self) -> u64 {
        self.0
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

    pub(crate) fn merge_entries(
        &mut self,
        target: AnimationTargetId,
        entries: &[crate::property::PropertyEntry],
    ) {
        if let Some(snapshot) = self.targets.get_mut(&target) {
            snapshot.merge_entries_unsorted(entries);
            snapshot.sort_by_composition_key();
        } else {
            let mut snapshot = PropertySnapshot::with_capacity(entries.len());
            snapshot.merge_entries_unsorted(entries);
            snapshot.sort_by_composition_key();
            self.order.push(target);
            self.targets.insert(target, snapshot);
        }
    }

    pub(crate) fn clear(&mut self) {
        self.targets.clear();
        self.order.clear();
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
