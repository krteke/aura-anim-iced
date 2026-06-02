use crate::{
    runtime::{AnimationClock, AnimationHandle, AnimationRuntime, AnimationTargetId},
    state::{
        ActiveStateTransition, StateTransition, StateTransitionProgress,
        StateTransitionRegistration, StateTransitionSet,
    },
    timeline::Timeline,
    timing::Duration,
};

use std::{hash::Hash, sync::Arc};

/// Tracks an application state and starts timelines for explicit state changes.
///
/// ```
/// use aura_anim_iced::{
///     property::OPACITY,
///     runtime::{AnimationRuntime, AnimationTargetId},
///     state::{StateAnimator, StateTransition},
///     timeline::{Timeline, Track},
///     timing::Duration,
/// };
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// enum Dialog {
///     Hidden,
///     Visible,
/// }
///
/// let mut runtime = AnimationRuntime::testing();
/// let target = AnimationTargetId::new();
/// let transition = StateTransition::new(
///     Dialog::Hidden,
///     Dialog::Visible,
///     Timeline::track(
///         Track::from(OPACITY, 0.0)
///             .to(1.0)
///             .duration(Duration::from_millis(200.0)),
///     ),
/// );
/// let mut animator = StateAnimator::new(target, Dialog::Hidden);
///
/// animator.transition_with(&mut runtime, &transition).unwrap();
/// let progress = animator
///     .active_progress_at(Duration::from_millis(50.0))
///     .expect("active state progress");
///
/// assert_eq!(animator.current(), Dialog::Visible);
/// assert_eq!(progress.progress(), Some(0.25));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct StateAnimator<S>
where
    S: Copy + Eq + Hash,
{
    target: AnimationTargetId,
    current: S,
    active: Option<ActiveStateTransition<S>>,
}

impl<S> StateAnimator<S>
where
    S: Copy + Eq + Hash,
{
    /// Creates a state animator for `target`.
    #[must_use]
    pub const fn new(target: AnimationTargetId, initial: S) -> Self {
        Self {
            target,
            current: initial,
            active: None,
        }
    }

    /// Returns the target that receives transition timelines.
    #[must_use]
    pub const fn target(&self) -> AnimationTargetId {
        self.target
    }

    /// Returns the latest application state observed by this animator.
    #[must_use]
    pub const fn current(&self) -> S {
        self.current
    }

    /// Returns the active runtime handle created by this animator, if any.
    #[must_use]
    pub const fn active_handle(&self) -> Option<AnimationHandle> {
        match self.active {
            Some(active) => Some(active.handle()),
            None => None,
        }
    }

    /// Returns whether this animator currently owns a runtime animation handle.
    #[must_use]
    pub fn is_active<C: AnimationClock>(&self, runtime: &AnimationRuntime<C>) -> bool {
        match self.active {
            Some(active) => runtime.contains(self.target, active.handle()),
            None => false,
        }
    }

    /// Returns metadata for the active state transition, if any.
    #[must_use]
    pub const fn active_transition(&self) -> Option<&ActiveStateTransition<S>> {
        self.active.as_ref()
    }

    /// Returns sampled progress for the active transition at `timestamp`.
    #[must_use]
    pub fn active_progress_at(&self, timestamp: Duration) -> Option<StateTransitionProgress<S>> {
        self.active
            .as_ref()
            .map(|active| active.progress_at(timestamp))
    }

    /// Refreshes active transition metadata when its runtime handle is gone.
    ///
    /// Transition start methods refresh stale active metadata automatically.
    /// Call this when application code needs the cached active transition state
    /// to be accurate before starting another transition.
    pub fn handle_completion<C: AnimationClock>(&mut self, runtime: &AnimationRuntime<C>) -> bool {
        let Some(active) = self.active else {
            return false;
        };

        if runtime.contains(self.target, active.handle()) {
            return false;
        }

        self.active = None;
        true
    }

    /// Starts `transition` when it matches the animator's current state.
    ///
    /// Returns `None` when `transition` does not start from the current state,
    /// or when it would keep the state unchanged.
    pub fn transition_with<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        transition: &StateTransition<S>,
    ) -> Option<StateTransitionRegistration<S>> {
        self.invalidate_if_stale(runtime);

        if self.current != transition.from() || transition.from() == transition.to() {
            return None;
        }

        Some(self.register_timeline(
            runtime,
            transition.from(),
            transition.to(),
            transition.timeline_arc(),
        ))
    }

    /// Finds and starts a transition from the current state to `to`.
    ///
    /// Uses the transition set fallback when no exact state-pair transition
    /// matches. Returns `None` when `to` is the current state or no transition
    /// behavior is available.
    pub fn transition_to<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        to: S,
        transitions: &StateTransitionSet<S>,
    ) -> Option<StateTransitionRegistration<S>> {
        self.invalidate_if_stale(runtime);

        if self.current == to {
            return None;
        }

        if let Some(transition) = transitions.find(self.current, to) {
            return self.transition_with(runtime, transition);
        }

        let fallback = transitions.fallback_arc()?;

        Some(self.register_timeline(runtime, self.current, to, fallback.clone()))
    }

    fn register_timeline<C: AnimationClock>(
        &mut self,
        runtime: &mut AnimationRuntime<C>,
        from: S,
        to: S,
        timeline: Arc<Timeline>,
    ) -> StateTransitionRegistration<S> {
        let replaced = self.active.take();

        let started_at = runtime.clock().now();
        let duration = timeline.total_duration();
        self.current = to;

        let registration = runtime.register_timeline_arc(self.target, timeline);

        self.active = Some(ActiveStateTransition::new(
            registration.handle(),
            from,
            to,
            started_at,
            duration,
        ));

        self.cleanup_replaced(runtime, replaced);

        StateTransitionRegistration::new(registration, replaced)
    }

    fn cleanup_replaced<C: AnimationClock>(
        &self,
        runtime: &mut AnimationRuntime<C>,
        replaced: Option<ActiveStateTransition<S>>,
    ) {
        if let Some(active) = replaced {
            runtime.cancel(self.target, active.handle());
        }
    }

    fn invalidate_if_stale<C: AnimationClock>(&mut self, runtime: &AnimationRuntime<C>) {
        if let Some(active) = self.active
            && !runtime.contains(self.target, active.handle())
        {
            self.active = None;
        }
    }
}
