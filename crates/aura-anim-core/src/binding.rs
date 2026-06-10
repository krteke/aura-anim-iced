//! Declarative business-state to motion bindings.

use std::sync::Arc;

mod error;

pub use error::MotionBindingError;

use crate::{
    Animatable, Animation, BoxAnimation, Motion, MotionRuntime, PlaybackId, Spring, SpringConfig,
    Tween, timing::Timing,
};

/// Owned values supplied to a [`MotionBinding`] transition factory.
///
/// `from` is always the motion's current sampled value, not the target
/// associated with `from_state`. This keeps interrupted transitions visually
/// continuous.
#[derive(Debug, Clone)]
pub struct TransitionContext<S, T> {
    /// Business state that was active before this transition.
    pub from_state: S,
    /// Business state being applied.
    pub to_state: S,
    /// Current sampled motion value.
    pub from: T,
    /// Target value associated with `to_state`.
    pub to: T,
}

impl<S, T: Animatable> TransitionContext<S, T> {
    /// Builds a tween from the current sampled value to the resolved target.
    #[must_use]
    pub fn tween(self, timing: Timing) -> Tween<T> {
        Tween::between(self.from, self.to, timing)
    }

    /// Builds a spring from the current sampled value to the resolved target.
    #[must_use]
    pub fn spring(self, config: SpringConfig) -> Spring<T> {
        Spring::new(self.from, self.to, config)
    }
}

/// Per-consumer state used with a reusable [`MotionBinding`].
///
/// Keep one value next to each bound [`Motion`]. The binding itself remains
/// immutable and can be shared by any number of controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MotionBindingState<S> {
    previous: S,
}

impl<S> MotionBindingState<S> {
    /// Creates state tracking from `previous`.
    #[must_use]
    pub const fn new(previous: S) -> Self {
        Self { previous }
    }

    /// Returns the last successfully applied business state.
    #[must_use]
    pub const fn current(&self) -> &S {
        &self.previous
    }
}

type TransitionFactory<S, T> = Arc<dyn Fn(TransitionContext<S, T>) -> BoxAnimation<T> + 'static>;

#[derive(Clone)]
struct Transition<S, T: Animatable> {
    from: S,
    to: S,
    factory: TransitionFactory<S, T>,
}

/// Reusable configuration that maps business states to motion targets.
///
/// A binding contains no mutable playback state. Call [`MotionBinding::state`]
/// once per consumer, then pass that state to [`MotionBinding::set_state`].
/// Exact `(from, to)` factories take precedence over the fallback factory.
///
/// # Examples
///
/// ```
/// use aura_anim_core::{MotionBinding, MotionRuntime, timing::{Duration, Timing}};
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// enum ButtonState {
///     Idle,
///     Hovered,
/// }
///
/// let binding = MotionBinding::new(ButtonState::Idle, 0.8_f32)
///     .when(ButtonState::Hovered, 1.0)
///     .transition(ButtonState::Idle, ButtonState::Hovered, |context| {
///         context.tween(Timing::ease_out(120.0))
///     })
///     .fallback(|context| context.tween(Timing::linear(100.0)));
///
/// let mut runtime = MotionRuntime::new();
/// let (motion, mut state) = binding.create_motion(&mut runtime);
/// let playback = binding
///     .set_state_tracked(
///         &mut state,
///         ButtonState::Hovered,
///         motion,
///         &mut runtime,
///     )
///     .unwrap()
///     .unwrap();
/// runtime.tick(Duration::from_millis(120.0));
///
/// assert_eq!(motion.value(&runtime).unwrap(), 1.0);
/// assert!(runtime.take_events()[0].is_completed_for(playback));
/// ```
#[derive(Clone)]
pub struct MotionBinding<S, T: Animatable> {
    initial_state: S,
    initial_target: T,
    targets: Vec<(S, T)>,
    transitions: Vec<Transition<S, T>>,
    fallback: Option<TransitionFactory<S, T>>,
}

impl<S, T> MotionBinding<S, T>
where
    S: Clone + PartialEq + 'static,
    T: Animatable,
{
    /// Creates a binding with its initial business state and target value.
    #[must_use]
    pub fn new(initial_state: S, initial_target: T) -> Self {
        Self {
            initial_state: initial_state.clone(),
            initial_target: initial_target.clone(),
            targets: vec![(initial_state, initial_target)],
            transitions: Vec::new(),
            fallback: None,
        }
    }

    /// Adds or replaces the target associated with `state`.
    #[must_use]
    pub fn when(mut self, state: S, target: T) -> Self {
        if state == self.initial_state {
            self.initial_target = target.clone();
        }
        if let Some((_, existing)) = self
            .targets
            .iter_mut()
            .find(|(existing, _)| existing == &state)
        {
            *existing = target;
        } else {
            self.targets.push((state, target));
        }
        self
    }

    /// Adds or replaces an exact `(from, to)` transition factory.
    ///
    /// The factory may return any concrete [`Animation`];
    /// the binding handles type erasure internally.
    #[must_use]
    pub fn transition<A>(
        mut self,
        from: S,
        to: S,
        factory: impl Fn(TransitionContext<S, T>) -> A + 'static,
    ) -> Self
    where
        A: Animation<T>,
    {
        let factory: TransitionFactory<S, T> = Arc::new(move |context| Box::new(factory(context)));
        if let Some(existing) = self
            .transitions
            .iter_mut()
            .find(|transition| transition.from == from && transition.to == to)
        {
            existing.factory = factory;
        } else {
            self.transitions.push(Transition { from, to, factory });
        }
        self
    }

    /// Sets the factory used when no exact transition is configured.
    ///
    /// The factory may return any concrete [`Animation`];
    /// the binding handles type erasure internally.
    #[must_use]
    pub fn fallback<A>(mut self, factory: impl Fn(TransitionContext<S, T>) -> A + 'static) -> Self
    where
        A: Animation<T>,
    {
        self.fallback = Some(Arc::new(move |context| Box::new(factory(context))));
        self
    }

    /// Returns the initial business state.
    #[must_use]
    pub const fn initial_state(&self) -> &S {
        &self.initial_state
    }

    /// Returns the target associated with `state`.
    pub fn target(&self, state: &S) -> Result<&T, MotionBindingError<S>> {
        let target = self
            .targets
            .iter()
            .find_map(|(candidate, target)| (candidate == state).then_some(target));

        #[cfg(feature = "tracing")]
        if target.is_none() {
            tracing::debug!(
                target: "aura_anim::binding",
                state_type = std::any::type_name::<S>(),
                value_type = std::any::type_name::<T>(),
                "motion binding target lookup failed"
            );
        }
        target.ok_or_else(|| MotionBindingError::MissingTarget(state.clone()))
    }

    /// Creates independent state tracking for one consumer.
    #[must_use]
    pub fn state(&self) -> MotionBindingState<S> {
        MotionBindingState::new(self.initial_state.clone())
    }

    /// Creates a motion at the initial target and its independent state tracker.
    pub fn create_motion(&self, runtime: &mut MotionRuntime) -> (Motion<T>, MotionBindingState<S>) {
        (runtime.motion(self.initial_target.clone()), self.state())
    }

    /// Applies `next_state` to an existing motion.
    ///
    /// Returns `Ok(false)` when the requested state is already current.
    /// Use [`MotionBinding::set_state_tracked`] when the concrete playback ID
    /// is needed for event matching.
    pub fn set_state(
        &self,
        binding_state: &mut MotionBindingState<S>,
        next_state: S,
        motion: Motion<T>,
        runtime: &mut MotionRuntime,
    ) -> Result<bool, MotionBindingError<S>> {
        self.set_state_tracked(binding_state, next_state, motion, runtime)
            .map(|playback| playback.is_some())
    }

    /// Applies `next_state` and returns the concrete playback ID.
    ///
    /// Returns `Ok(None)` when the requested state is already current.
    /// Target and transition lookup happen before the motion is modified.
    /// The state tracker is updated only after `motion.play_tracked` succeeds.
    pub fn set_state_tracked(
        &self,
        binding_state: &mut MotionBindingState<S>,
        next_state: S,
        motion: Motion<T>,
        runtime: &mut MotionRuntime,
    ) -> Result<Option<PlaybackId>, MotionBindingError<S>> {
        if binding_state.previous == next_state {
            #[cfg(feature = "tracing")]
            tracing::trace!(
                target: "aura_anim::binding",
                state_type = std::any::type_name::<S>(),
                value_type = std::any::type_name::<T>(),
                "motion binding state is unchanged"
            );
            return Ok(None);
        }

        let target = self.target(&next_state).cloned()?;
        let previous = binding_state.previous.clone();
        let exact_factory = self
            .transitions
            .iter()
            .find(|transition| transition.from == previous && transition.to == next_state)
            .map(|transition| &transition.factory);
        let Some(factory) = exact_factory.or(self.fallback.as_ref()) else {
            #[cfg(feature = "tracing")]
            tracing::debug!(
                target: "aura_anim::binding",
                state_type = std::any::type_name::<S>(),
                value_type = std::any::type_name::<T>(),
                "motion binding transition lookup failed"
            );
            return Err(MotionBindingError::MissingTransition {
                from: previous.clone(),
                to: next_state.clone(),
            });
        };
        let current = motion.value(runtime)?;
        #[cfg(feature = "tracing")]
        {
            let uses_fallback = exact_factory.is_none();
            tracing::debug!(
                target: "aura_anim::binding",
                state_type = std::any::type_name::<S>(),
                value_type = std::any::type_name::<T>(),
                uses_fallback,
                "applying motion binding transition"
            );
            let _ = uses_fallback;
        }
        let animation = factory(TransitionContext {
            from_state: previous,
            to_state: next_state.clone(),
            from: current,
            to: target,
        });

        let playback = motion.play_tracked(animation, runtime)?;

        binding_state.previous = next_state;
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "aura_anim::binding",
            state_type = std::any::type_name::<S>(),
            value_type = std::any::type_name::<T>(),
            "committed motion binding state"
        );
        Ok(Some(playback))
    }
}

#[cfg(test)]
mod tests {
    use super::{MotionBinding, MotionBindingError};
    use crate::{
        AnimationExt, MotionRuntime, Sequence, SpringConfig, Tween,
        keyframes::Keyframes,
        timing::{Duration, Timing},
    };
    use float_cmp::assert_approx_eq;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum State {
        Idle,
        Hovered,
        Pressed,
        Disabled,
    }

    fn binding() -> MotionBinding<State, f32> {
        MotionBinding::new(State::Idle, 0.0)
            .when(State::Hovered, 10.0)
            .when(State::Pressed, 20.0)
            .transition(State::Idle, State::Hovered, |context| {
                context.tween(Timing::new(100.0))
            })
            .transition(State::Hovered, State::Pressed, |context| {
                context.spring(SpringConfig::default())
            })
            .fallback(|context| context.tween(Timing::new(50.0)))
    }

    #[test]
    fn exact_transition_uses_current_sampled_value() {
        let binding = binding();
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);

        binding
            .set_state(&mut state, State::Hovered, motion, &mut runtime)
            .unwrap();
        runtime.tick(Duration::from_millis(40.0));
        assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 4.0);

        binding
            .set_state(&mut state, State::Pressed, motion, &mut runtime)
            .unwrap();

        assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 4.0);
        assert_eq!(state.current(), &State::Pressed);
    }

    #[test]
    fn fallback_handles_unlisted_state_pair() {
        let binding = binding();
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);

        binding
            .set_state(&mut state, State::Pressed, motion, &mut runtime)
            .unwrap();
        runtime.tick(Duration::from_millis(50.0));

        assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 20.0);
    }

    #[test]
    fn tracked_state_change_returns_playback_for_completion_event() {
        let binding = binding();
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);

        let playback = binding
            .set_state_tracked(&mut state, State::Hovered, motion, &mut runtime)
            .unwrap()
            .unwrap();

        assert_eq!(motion.playback(&runtime).unwrap(), playback);
        assert_eq!(
            binding
                .set_state_tracked(&mut state, State::Hovered, motion, &mut runtime)
                .unwrap(),
            None
        );

        runtime.tick(Duration::from_millis(100.0));
        let events = runtime.take_events();
        assert_eq!(events.len(), 1);
        assert!(events[0].is_completed_for(playback));
    }

    #[test]
    fn failed_lookup_does_not_commit_state() {
        let binding = binding();
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);

        let error = binding
            .set_state(&mut state, State::Disabled, motion, &mut runtime)
            .unwrap_err();

        assert_eq!(error, MotionBindingError::MissingTarget(State::Disabled));
        assert_eq!(state.current(), &State::Idle);
    }

    #[test]
    fn shared_configuration_creates_independent_trackers() {
        let binding = binding();
        let mut runtime = MotionRuntime::new();
        let (first, mut first_state) = binding.create_motion(&mut runtime);
        let (second, second_state) = binding.create_motion(&mut runtime);

        binding
            .set_state(&mut first_state, State::Hovered, first, &mut runtime)
            .unwrap();

        assert_eq!(first_state.current(), &State::Hovered);
        assert_eq!(second_state.current(), &State::Idle);
        assert_approx_eq!(f32, second.value(&runtime).unwrap(), 0.0);
    }

    #[test]
    fn missing_transition_is_reported_before_playback() {
        let binding = MotionBinding::new(State::Idle, 0.0).when(State::Hovered, 1.0);
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);

        let error = binding
            .set_state(&mut state, State::Hovered, motion, &mut runtime)
            .unwrap_err();

        assert_eq!(
            error,
            MotionBindingError::MissingTransition {
                from: State::Idle,
                to: State::Hovered,
            }
        );
        assert!(!motion.is_active(&runtime).unwrap());
    }

    #[test]
    fn runtime_errors_are_preserved_by_binding_errors() {
        let binding = binding();
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);
        motion.remove(&mut runtime).unwrap();

        let error = binding
            .set_state(&mut state, State::Hovered, motion, &mut runtime)
            .unwrap_err();

        assert_eq!(
            error,
            MotionBindingError::Motion(crate::MotionError::Removed { slot: 0 })
        );
        assert_eq!(state.current(), &State::Idle);
    }

    #[test]
    fn factories_accept_keyframes_and_timeline_sources() {
        let binding = MotionBinding::new(State::Idle, 0.0_f32)
            .when(State::Hovered, 10.0)
            .when(State::Pressed, 20.0)
            .transition(State::Idle, State::Hovered, |context| {
                Keyframes::new(context.from).push(100.0, context.to)
            })
            .transition(State::Hovered, State::Pressed, |context| {
                Sequence::new(context.from).then(Tween::between(
                    context.from,
                    context.to,
                    Timing::new(100.0),
                ))
            });
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);

        binding
            .set_state(&mut state, State::Hovered, motion, &mut runtime)
            .unwrap();
        runtime.tick(Duration::from_millis(100.0));
        assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 10.0);

        binding
            .set_state(&mut state, State::Pressed, motion, &mut runtime)
            .unwrap();
        runtime.tick(Duration::from_millis(100.0));
        assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 20.0);
    }

    #[test]
    fn boxed_factories_remain_supported() {
        let binding = MotionBinding::new(State::Idle, 0.0_f32)
            .when(State::Hovered, 10.0)
            .fallback(|context| context.tween(Timing::new(100.0)).boxed());
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);

        binding
            .set_state(&mut state, State::Hovered, motion, &mut runtime)
            .unwrap();
        runtime.tick(Duration::from_millis(100.0));

        assert_approx_eq!(f32, motion.value(&runtime).unwrap(), 10.0);
    }
}
