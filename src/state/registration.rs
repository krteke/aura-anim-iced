use crate::{ActiveStateTransition, AnimationHandle, AnimationRegistration};

/// Output produced when a state transition is registered.
#[derive(Debug, Clone, PartialEq)]
pub struct StateTransitionRegistration<S>
where
    S: Copy + Eq,
{
    registration: AnimationRegistration,
    replaced: Option<ActiveStateTransition<S>>,
}

impl<S> StateTransitionRegistration<S>
where
    S: Copy + Eq,
{
    pub(crate) const fn new(
        registration: AnimationRegistration,
        replaced: Option<ActiveStateTransition<S>>,
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

    /// Returns the active state transition replaced by this registration.
    #[must_use]
    pub const fn replaced(&self) -> Option<&ActiveStateTransition<S>> {
        self.replaced.as_ref()
    }

    /// Converts this value into its runtime animation registration.
    #[must_use]
    pub fn into_registration(self) -> AnimationRegistration {
        self.registration
    }
}
