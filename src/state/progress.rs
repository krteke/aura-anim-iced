use crate::{AnimationHandle, Duration};

/// Active state transition metadata tracked by a [`StateAnimator`](crate::StateAnimator).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveStateTransition<S>
where
    S: Copy + Eq,
{
    handle: AnimationHandle,
    from: S,
    to: S,
    started_at: Duration,
    duration: Option<Duration>,
}

impl<S> ActiveStateTransition<S>
where
    S: Copy + Eq,
{
    pub(crate) const fn new(
        handle: AnimationHandle,
        from: S,
        to: S,
        started_at: Duration,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            handle,
            from,
            to,
            started_at,
            duration,
        }
    }

    /// Returns the runtime handle for this transition.
    #[must_use]
    pub const fn handle(&self) -> AnimationHandle {
        self.handle
    }

    /// Returns the state this transition started from.
    #[must_use]
    pub const fn from(&self) -> S {
        self.from
    }

    /// Returns the state this transition targets.
    #[must_use]
    pub const fn to(&self) -> S {
        self.to
    }

    /// Returns the runtime timestamp when this transition started.
    #[must_use]
    pub const fn started_at(&self) -> Duration {
        self.started_at
    }

    /// Returns the finite transition duration, if known.
    #[must_use]
    pub const fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Samples progress for this transition at a runtime timestamp.
    #[must_use]
    pub fn progress_at(&self, timestamp: Duration) -> StateTransitionProgress<S> {
        let elapsed = timestamp
            .checked_sub(self.started_at)
            .unwrap_or(Duration::ZERO);
        let progress = self
            .duration
            .map(|duration| normalized_progress(elapsed, duration));

        StateTransitionProgress {
            handle: self.handle,
            from: self.from,
            to: self.to,
            elapsed,
            duration: self.duration,
            progress,
        }
    }
}

/// Sampled progress for an active state transition.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateTransitionProgress<S>
where
    S: Copy + Eq,
{
    handle: AnimationHandle,
    from: S,
    to: S,
    elapsed: Duration,
    duration: Option<Duration>,
    progress: Option<f32>,
}

impl<S> StateTransitionProgress<S>
where
    S: Copy + Eq,
{
    /// Returns the runtime handle for this transition.
    #[must_use]
    pub const fn handle(&self) -> AnimationHandle {
        self.handle
    }

    /// Returns the state this transition started from.
    #[must_use]
    pub const fn from(&self) -> S {
        self.from
    }

    /// Returns the state this transition targets.
    #[must_use]
    pub const fn to(&self) -> S {
        self.to
    }

    /// Returns elapsed runtime time since the transition started.
    #[must_use]
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns the finite transition duration, if known.
    #[must_use]
    pub const fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Returns normalized progress in `[0.0, 1.0]` for finite transitions.
    #[must_use]
    pub const fn progress(&self) -> Option<f32> {
        self.progress
    }
}

fn normalized_progress(elapsed: Duration, duration: Duration) -> f32 {
    let duration_ms = duration.as_millis();

    if duration_ms <= 0.0 {
        return 1.0;
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "State transition progress is normalized for UI consumption."
    )]
    {
        (elapsed.as_millis() / duration_ms).clamp(0.0, 1.0) as f32
    }
}
