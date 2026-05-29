use std::collections::HashMap;

use crate::{
    Duration,
    runtime::{entry::ActiveAnimation, target::AnimationTargetId},
};

use super::AnimationHandle;

/// Internal storage for active runtime animation entries.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AnimationRegistry {
    entries: Vec<ActiveAnimation>,
    next_handle_id: u64,
    target_map: HashMap<AnimationTargetId, Vec<AnimationHandle>>,
}

impl AnimationRegistry {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_handle_id: AnimationHandle::FIRST_ID,
            target_map: HashMap::new(),
        }
    }

    #[must_use]
    pub(crate) fn active_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.is_active())
            .count()
    }

    #[must_use]
    pub(crate) fn entries(&self) -> &[ActiveAnimation] {
        &self.entries
    }

    #[must_use]
    pub(crate) fn entries_mut(&mut self) -> &mut [ActiveAnimation] {
        &mut self.entries
    }

    #[must_use]
    pub(crate) fn allocate_handle(&mut self) -> AnimationHandle {
        let handle = AnimationHandle::new(self.next_handle_id);
        self.next_handle_id = self.next_handle_id.saturating_add(1);
        handle
    }

    pub(crate) fn insert(
        &mut self,
        target: AnimationTargetId,
        entry: ActiveAnimation,
    ) -> AnimationHandle {
        let handle = entry.handle();
        debug_assert_eq!(target, entry.target());
        self.entries.push(entry);
        self.target_map.entry(target).or_default().push(handle);
        handle
    }

    #[must_use]
    pub(crate) fn get_by_handle(&self, handle: AnimationHandle) -> Option<&ActiveAnimation> {
        self.entries.iter().find(|entry| entry.handle() == handle)
    }

    #[must_use]
    pub(crate) fn get_mut_by_handle(
        &mut self,
        handle: AnimationHandle,
    ) -> Option<&mut ActiveAnimation> {
        self.entries
            .iter_mut()
            .find(|entry| entry.handle() == handle)
    }

    pub(crate) fn remove_by_handle(&mut self, handle: AnimationHandle) -> Option<ActiveAnimation> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.handle() == handle)?;

        let removed = self.entries.remove(index);
        self.remove_handle_from_target_map(removed.target(), handle);

        Some(removed)
    }

    pub(crate) fn cancel_target(&mut self, target: AnimationTargetId) {
        self.entries.retain(|entry| entry.target() != target);
        self.target_map.remove(&target);
    }

    pub(crate) fn seek_target(&mut self, target: AnimationTargetId, pos: Duration, now: Duration) {
        if let Some(handles) = self.target_map.get(&target).cloned() {
            for handle in handles {
                self.seek(target, handle, pos, now);
            }
        }
    }

    pub(crate) fn pause_target(&mut self, target: AnimationTargetId, now: Duration) {
        if let Some(handles) = self.target_map.get(&target).cloned() {
            for handle in handles {
                self.pause(target, handle, now);
            }
        }
    }

    pub(crate) fn cancel(&mut self, target: AnimationTargetId, handle: AnimationHandle) -> bool {
        let Some(entry) = self.get_by_handle(handle) else {
            return false;
        };

        if entry.target() != target {
            return false;
        }

        self.remove_by_handle(handle).is_some()
    }

    pub(crate) fn seek(
        &mut self,
        target: AnimationTargetId,
        handle: AnimationHandle,
        pos: Duration,
        now: Duration,
    ) -> bool {
        let Some(entry) = self.get_mut_by_handle(handle) else {
            return false;
        };

        if entry.target() != target {
            return false;
        }

        entry.set_position(pos);
        entry.set_last_tick(now);
        entry.set_last_snapshot(entry.source().sample_at(pos));

        true
    }

    pub(crate) fn pause(
        &mut self,
        target: AnimationTargetId,
        handle: AnimationHandle,
        now: Duration,
    ) -> bool {
        let Some(entry) = self.get_mut_by_handle(handle) else {
            return false;
        };

        if entry.target() != target {
            return false;
        }

        entry.set_state(super::AnimationPlaybackState::Paused);
        entry.set_last_tick(now);

        true
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
