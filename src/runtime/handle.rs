use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(AnimationHandle::FIRST_ID);

/// Stable identifier for an animation stored in the runtime registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AnimationHandle(u64);

impl AnimationHandle {
    pub(super) const FIRST_ID: u64 = 1;

    pub(super) fn new() -> Self {
        Self(NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the numeric handle ID.
    #[must_use]
    pub const fn id(self) -> u64 {
        self.0
    }
}
