//! Errors returned by typed motion handles and runtime operations.

/// Failure while accessing or modifying a runtime-managed motion.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MotionError {
    /// The handle references a slot that does not exist.
    #[error("motion slot {slot} is outside the runtime storage")]
    SlotOutOfBounds {
        /// Referenced slot index.
        slot: usize,
    },
    /// The slot has been reused since this handle was created.
    #[error(
        "motion slot {slot} has generation {actual_generation}, but the handle expects {handle_generation}"
    )]
    StaleHandle {
        /// Referenced slot index.
        slot: usize,
        /// Generation stored in the handle.
        handle_generation: u64,
        /// Current generation stored in the runtime.
        actual_generation: u64,
    },
    /// The animation was removed from its slot.
    #[error("motion slot {slot} no longer contains an animation")]
    Removed {
        /// Referenced slot index.
        slot: usize,
    },
    /// The handle's value type does not match the animation in the slot.
    #[error("motion value type mismatch: expected {expected}, found {actual}")]
    TypeMismatch {
        /// Type requested through the handle.
        expected: &'static str,
        /// Type currently stored in the slot.
        actual: &'static str,
    },
}
