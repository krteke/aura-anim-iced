use std::sync::atomic::AtomicU64;

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(AnimationHandle::FIRST);

#[derive(Debug, Clone, Copy)]
pub struct AnimationHandle(u64);

impl AnimationHandle {
    const FIRST: u64 = 0;

    pub fn new() -> Self {
        Self(NEXT_HANDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }

    pub const fn id(&self) -> u64 {
        self.0
    }
}

impl Default for AnimationHandle {
    fn default() -> Self {
        Self::new()
    }
}
