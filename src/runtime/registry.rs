use std::collections::HashMap;

use crate::{Duration, runtime::target::AnimationTargetId};

use super::{ActiveAnimation, AnimationHandle};

/// Storage for active runtime animation entries.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationRegistry {
    entries: Vec<ActiveAnimation>,
    next_handle_id: u64,
    target_map: HashMap<AnimationTargetId, Vec<AnimationHandle>>,
}

impl AnimationRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_handle_id: AnimationHandle::FIRST_ID,
            target_map: HashMap::new(),
        }
    }

    /// Returns the number of active entries.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.is_active())
            .count()
    }

    /// Returns whether the registry has no active entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns active entries in insertion order.
    #[must_use]
    pub fn entries(&self) -> &[ActiveAnimation] {
        &self.entries
    }

    /// Returns mutable active entries in insertion order.
    #[must_use]
    pub fn entries_mut(&mut self) -> &mut [ActiveAnimation] {
        &mut self.entries
    }

    /// Allocates a stable handle.
    #[must_use]
    pub fn allocate_handle(&mut self) -> AnimationHandle {
        let handle = AnimationHandle::new(self.next_handle_id);
        self.next_handle_id = self.next_handle_id.saturating_add(1);
        handle
    }

    pub fn insert(&mut self, target: AnimationTargetId, entry: ActiveAnimation) -> AnimationHandle {
        let handle = entry.handle();
        debug_assert_eq!(target, entry.target());
        self.entries.push(entry);
        self.target_map.entry(target).or_default().push(handle);
        handle
    }

    /// Returns an active entry by handle.
    #[must_use]
    pub fn get_by_handle(&self, handle: AnimationHandle) -> Option<&ActiveAnimation> {
        self.entries.iter().find(|entry| entry.handle() == handle)
    }

    /// Returns mutable access to an active entry by handle.
    #[must_use]
    pub fn get_mut_by_handle(&mut self, handle: AnimationHandle) -> Option<&mut ActiveAnimation> {
        self.entries
            .iter_mut()
            .find(|entry| entry.handle() == handle)
    }

    /// Removes an active entry by handle.
    pub fn remove_by_handle(&mut self, handle: AnimationHandle) -> Option<ActiveAnimation> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.handle() == handle)?;

        let removed = self.entries.remove(index);
        self.remove_handle_from_target_map(removed.target(), handle);

        Some(removed)
    }

    /// Removes all active entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.target_map.clear();
    }

    pub(crate) fn cancel_target(&mut self, target: AnimationTargetId) {
        if let Some(old) = self.target_map.get(&target) {
            for handle in old {
                self.entries.retain(|entry| entry.handle() != *handle);
            }
        }
        self.target_map.remove(&target);
    }

    pub(crate) fn seek_target(&mut self, target: AnimationTargetId, pos: Duration, now: Duration) {
        if let Some(handles) = self.target_map.get_mut(&target) {
            for handle in handles {
                if let Some(entry) = self
                    .entries
                    .iter_mut()
                    .find(|entry| entry.handle() == *handle)
                {
                    entry.set_position(pos);
                    entry.set_last_tick(now);
                    entry.set_last_snapshot(entry.source().sample_at(pos));
                }
            }
        }
    }

    fn remove_handle_from_target_map(
        &mut self,
        target: AnimationTargetId,
        handle: AnimationHandle,
    ) {
        if let Some(handles) = self.target_map.get_mut(&target) {
            handles.retain(|candidate| *candidate != handle);
            if handles.is_empty() {
                self.target_map.remove(&target);
            }
        }
    }
}

impl Default for AnimationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
