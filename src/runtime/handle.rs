/// Stable identifier for an animation stored in the runtime registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AnimationHandle(u64);

impl AnimationHandle {
    pub(super) const FIRST_ID: u64 = 1;

    pub(super) const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the numeric handle ID.
    #[must_use]
    pub const fn id(self) -> u64 {
        self.0
    }
}
