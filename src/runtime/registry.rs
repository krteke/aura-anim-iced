use super::{ActiveAnimation, AnimationHandle};

/// Storage for active runtime animation entries.
#[derive(Debug, Clone, PartialEq)]
pub struct AnimationRegistry {
    entries: Vec<ActiveAnimation>,
    next_handle_id: u64,
}

impl AnimationRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_handle_id: AnimationHandle::FIRST_ID,
        }
    }

    /// Returns the number of active entries.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.entries.len()
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

    /// Inserts an active entry and returns its handle.
    pub fn insert(&mut self, entry: ActiveAnimation) -> AnimationHandle {
        let handle = entry.handle();
        self.entries.push(entry);
        handle
    }

    /// Returns an active entry by handle.
    #[must_use]
    pub fn get(&self, handle: AnimationHandle) -> Option<&ActiveAnimation> {
        self.entries.iter().find(|entry| entry.handle() == handle)
    }

    /// Returns mutable access to an active entry by handle.
    #[must_use]
    pub fn get_mut(&mut self, handle: AnimationHandle) -> Option<&mut ActiveAnimation> {
        self.entries
            .iter_mut()
            .find(|entry| entry.handle() == handle)
    }

    /// Removes an active entry by handle.
    pub fn remove(&mut self, handle: AnimationHandle) -> Option<ActiveAnimation> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.handle() == handle)?;

        Some(self.entries.remove(index))
    }

    /// Removes all active entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for AnimationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
