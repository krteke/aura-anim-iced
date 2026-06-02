use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use crate::{
    runtime::{entry::ActiveAnimation, target::AnimationTargetId},
    timing::Duration,
};

use super::AnimationHandle;

/// Internal storage for active runtime animation entries.
#[derive(Debug, Clone)]
pub(crate) struct AnimationRegistry {
    entries: Vec<ActiveAnimation>,
    handle_to_index: FxHashMap<AnimationHandle, usize>,
    target_map: FxHashMap<AnimationTargetId, SmallVec<[AnimationHandle; 4]>>,
}

impl AnimationRegistry {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            entries: Vec::new(),
            handle_to_index: FxHashMap::default(),
            target_map: FxHashMap::default(),
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

    pub(crate) fn insert(
        &mut self,
        target: AnimationTargetId,
        entry: ActiveAnimation,
    ) -> AnimationHandle {
        let handle = entry.handle();
        debug_assert_eq!(target, entry.target());

        let index = self.entries.len();
        self.entries.push(entry);
        self.handle_to_index.insert(handle, index);
        self.target_map.entry(target).or_default().push(handle);

        handle
    }

    pub(crate) fn get_by_handle(&self, handle: AnimationHandle) -> Option<&ActiveAnimation> {
        let index = self.handle_to_index.get(&handle)?;
        self.entries.get(*index)
    }

    #[must_use]
    pub(crate) fn get_mut_by_handle(
        &mut self,
        handle: AnimationHandle,
    ) -> Option<&mut ActiveAnimation> {
        let index = self.handle_to_index.get(&handle)?;
        self.entries.get_mut(*index)
    }

    pub(crate) fn remove_by_handle(&mut self, handle: AnimationHandle) -> Option<ActiveAnimation> {
        let index = self.handle_to_index.remove(&handle)?;
        let removed = self.entries.swap_remove(index);

        if index < self.entries.len() {
            let moved = self.entries[index].handle();
            self.handle_to_index.insert(moved, index);
        }

        self.remove_handle_from_target_map(removed.target(), handle);

        Some(removed)
    }

    pub(crate) fn cancel_target(&mut self, target: AnimationTargetId) {
        let handles = self.target_map.remove(&target);
        let Some(handles) = handles else {
            return;
        };

        for handle in handles {
            self.remove_by_handle(handle);
        }
    }

    pub(crate) fn seek_target(&mut self, target: AnimationTargetId, pos: Duration, now: Duration) {
        let Some(handles) = self.target_map.get(&target).cloned() else {
            return;
        };

        for handle in handles {
            self.seek(target, handle, pos, now);
        }
    }

    pub(crate) fn pause_target(&mut self, target: AnimationTargetId, now: Duration) {
        let Some(handles) = self.target_map.get(&target).cloned() else {
            return;
        };

        for handle in handles {
            self.pause(target, handle, now);
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
        let Some(handles) = self.target_map.get_mut(&target) else {
            return;
        };

        handles.retain(|candidate| *candidate != handle);
        if handles.is_empty() {
            self.target_map.remove(&target);
        }
    }
}

impl Default for AnimationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
