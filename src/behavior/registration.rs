use crate::runtime::{AnimationHandle, AnimationRegistration};

/// Output produced when a property transition is registered.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTransitionRegistration {
    registration: AnimationRegistration,
    replaced: Option<AnimationHandle>,
}

impl PropertyTransitionRegistration {
    pub(crate) const fn new(
        registration: AnimationRegistration,
        replaced: Option<AnimationHandle>,
    ) -> Self {
        Self {
            registration,
            replaced,
        }
    }

    /// Returns the runtime animation registration.
    #[must_use]
    pub const fn registration(&self) -> &AnimationRegistration {
        &self.registration
    }

    /// Returns the registered runtime handle.
    #[must_use]
    pub const fn handle(&self) -> AnimationHandle {
        self.registration.handle()
    }

    /// Returns the runtime handle replaced by this registration.
    #[must_use]
    pub const fn replaced(&self) -> Option<AnimationHandle> {
        self.replaced
    }

    /// Converts this value into its runtime animation registration.
    #[must_use]
    pub fn into_registration(self) -> AnimationRegistration {
        self.registration
    }
}
