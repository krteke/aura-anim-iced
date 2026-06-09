//! Declarative business-state to motion bindings.

use std::{error::Error, fmt, sync::Arc};

use crate::{Animatable, BoxAnimation, Motion, MotionRuntime};

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

/// Failure returned while applying a motion binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotionBindingError<S> {
    /// The requested business state has no target value.
    MissingTarget(S),
    /// Neither an exact transition nor a fallback factory was configured.
    MissingTransition {
        /// Previously applied state.
        from: S,
        /// Requested state.
        to: S,
    },
    /// The supplied motion handle is no longer valid.
    InvalidMotion,
}

impl<S: fmt::Debug> fmt::Display for MotionBindingError<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTarget(state) => {
                write!(
                    formatter,
                    "motion binding has no target for state {state:?}"
                )
            }
            Self::MissingTransition { from, to } => write!(
                formatter,
                "motion binding has no transition from {from:?} to {to:?}"
            ),
            Self::InvalidMotion => formatter.write_str("motion handle is no longer valid"),
        }
    }
}

impl<S: fmt::Debug> Error for MotionBindingError<S> {}

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
/// use aura_anim_core::{
///     AnimationExt, MotionBinding, MotionRuntime, Tween,
///     timing::{Duration, Timing},
/// };
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
///         Tween::between(context.from, context.to, Timing::new(120.0)).boxed()
///     })
///     .fallback(|context| {
///         Tween::between(context.from, context.to, Timing::new(100.0)).boxed()
///     });
///
/// let mut runtime = MotionRuntime::new();
/// let (motion, mut state) = binding.create_motion(&mut runtime);
/// binding
///     .set_state(
///         &mut state,
///         ButtonState::Hovered,
///         motion,
///         &mut runtime,
///     )
///     .unwrap();
/// runtime.tick(Duration::from_millis(120.0));
///
/// assert_eq!(motion.value(&runtime), 1.0);
/// ```
#[derive(Clone)]
pub struct MotionBinding<S, T: Animatable> {
    initial_state: S,
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
            targets: vec![(initial_state, initial_target)],
            transitions: Vec::new(),
            fallback: None,
        }
    }

    /// Adds or replaces the target associated with `state`.
    #[must_use]
    pub fn when(mut self, state: S, target: T) -> Self {
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
    #[must_use]
    pub fn transition(
        mut self,
        from: S,
        to: S,
        factory: impl Fn(TransitionContext<S, T>) -> BoxAnimation<T> + 'static,
    ) -> Self {
        let factory = Arc::new(factory);
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
    #[must_use]
    pub fn fallback(
        mut self,
        factory: impl Fn(TransitionContext<S, T>) -> BoxAnimation<T> + 'static,
    ) -> Self {
        self.fallback = Some(Arc::new(factory));
        self
    }

    /// Returns the initial business state.
    #[must_use]
    pub const fn initial_state(&self) -> &S {
        &self.initial_state
    }

    /// Returns the target associated with `state`.
    #[must_use]
    pub fn target(&self, state: &S) -> Option<&T> {
        self.targets
            .iter()
            .find_map(|(candidate, target)| (candidate == state).then_some(target))
    }

    /// Creates independent state tracking for one consumer.
    #[must_use]
    pub fn state(&self) -> MotionBindingState<S> {
        MotionBindingState::new(self.initial_state.clone())
    }

    /// Creates a motion at the initial target and its independent state tracker.
    pub fn create_motion(&self, runtime: &mut MotionRuntime) -> (Motion<T>, MotionBindingState<S>) {
        let target = self
            .target(&self.initial_state)
            .expect("MotionBinding::new always stores the initial target")
            .clone();
        (runtime.motion(target), self.state())
    }

    /// Applies `next_state` to an existing motion.
    ///
    /// Returns `Ok(false)` when the requested state is already current.
    /// Target and transition lookup happen before the motion is modified.
    /// The state tracker is updated only after `motion.play` succeeds.
    pub fn set_state(
        &self,
        binding_state: &mut MotionBindingState<S>,
        next_state: S,
        motion: Motion<T>,
        runtime: &mut MotionRuntime,
    ) -> Result<bool, MotionBindingError<S>> {
        if binding_state.previous == next_state {
            return Ok(false);
        }

        let target = self
            .target(&next_state)
            .cloned()
            .ok_or_else(|| MotionBindingError::MissingTarget(next_state.clone()))?;
        let previous = binding_state.previous.clone();
        let factory = self
            .transitions
            .iter()
            .find(|transition| transition.from == previous && transition.to == next_state)
            .map(|transition| &transition.factory)
            .or(self.fallback.as_ref())
            .ok_or_else(|| MotionBindingError::MissingTransition {
                from: previous.clone(),
                to: next_state.clone(),
            })?;
        let current = motion
            .try_value(runtime)
            .cloned()
            .ok_or(MotionBindingError::InvalidMotion)?;
        let animation = factory(TransitionContext {
            from_state: previous,
            to_state: next_state.clone(),
            from: current,
            to: target,
        });

        if !motion.play(animation, runtime) {
            return Err(MotionBindingError::InvalidMotion);
        }

        binding_state.previous = next_state;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::{MotionBinding, MotionBindingError};
    use crate::{
        AnimationExt, MotionRuntime, Sequence, Spring, SpringConfig, Tween,
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
                Tween::between(context.from, context.to, Timing::new(100.0)).boxed()
            })
            .transition(State::Hovered, State::Pressed, |context| {
                Spring::new(context.from, context.to, SpringConfig::default()).boxed()
            })
            .fallback(|context| Tween::between(context.from, context.to, Timing::new(50.0)).boxed())
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
        assert_approx_eq!(f32, motion.value(&runtime), 4.0);

        binding
            .set_state(&mut state, State::Pressed, motion, &mut runtime)
            .unwrap();

        assert_approx_eq!(f32, motion.value(&runtime), 4.0);
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

        assert_approx_eq!(f32, motion.value(&runtime), 20.0);
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
        assert_approx_eq!(f32, second.value(&runtime), 0.0);
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
        assert!(!motion.is_active(&runtime));
    }

    #[test]
    fn factories_accept_keyframes_and_timeline_sources() {
        let binding = MotionBinding::new(State::Idle, 0.0_f32)
            .when(State::Hovered, 10.0)
            .when(State::Pressed, 20.0)
            .transition(State::Idle, State::Hovered, |context| {
                Keyframes::new(context.from).push(100.0, context.to).boxed()
            })
            .transition(State::Hovered, State::Pressed, |context| {
                Sequence::new(context.from)
                    .then(Tween::between(context.from, context.to, Timing::new(100.0)))
                    .boxed()
            });
        let mut runtime = MotionRuntime::new();
        let (motion, mut state) = binding.create_motion(&mut runtime);

        binding
            .set_state(&mut state, State::Hovered, motion, &mut runtime)
            .unwrap();
        runtime.tick(Duration::from_millis(100.0));
        assert_approx_eq!(f32, motion.value(&runtime), 10.0);

        binding
            .set_state(&mut state, State::Pressed, motion, &mut runtime)
            .unwrap();
        runtime.tick(Duration::from_millis(100.0));
        assert_approx_eq!(f32, motion.value(&runtime), 20.0);
    }
}
