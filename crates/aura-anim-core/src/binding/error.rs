use crate::MotionError;

/// Failure returned while applying a motion binding.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum MotionBindingError<S> {
    /// The requested business state has no target value.
    #[error("motion binding has no target for state {0:?}")]
    MissingTarget(S),
    /// Neither an exact transition nor a fallback factory was configured.
    #[error("motion binding has no transition from {from:?} to {to:?}")]
    MissingTransition {
        /// Previously applied state.
        from: S,
        /// Requested state.
        to: S,
    },
    /// The runtime rejected the supplied motion handle.
    #[error(transparent)]
    Motion(#[from] MotionError),
}
