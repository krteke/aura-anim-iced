//! Spring-based animations with independently configured physical channels.

use std::{fmt, sync::Arc};

use crate::{
    timing::Duration,
    traits::{Animatable, Animation, AnimationState},
};

/// Physical parameters used by a [`Spring`] channel.
///
/// Invalid parameters are sanitized when a spring is created: stiffness and
/// damping are clamped to non-negative finite values, mass is clamped to a
/// positive finite value, and epsilon falls back to the default threshold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    /// Restoring force applied toward the target.
    pub stiffness: f32,
    /// Resistance applied against the current velocity.
    pub damping: f32,
    /// Inertial mass used by the spring simulation.
    pub mass: f32,
    /// Position and velocity threshold used to detect completion.
    pub epsilon: f32,
}

impl SpringConfig {
    fn sanitized(self) -> Self {
        let defaults = Self::default();
        Self {
            stiffness: finite_non_negative(self.stiffness),
            damping: finite_non_negative(self.damping),
            mass: if self.mass.is_finite() && self.mass > 0.0 {
                self.mass
            } else {
                f32::EPSILON
            },
            epsilon: if self.epsilon.is_finite() && self.epsilon > 0.0 {
                self.epsilon
            } else {
                defaults.epsilon
            },
        }
    }
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: 220.0,
            damping: 24.0,
            mass: 1.0,
            epsilon: 0.001,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SpringChannel {
    position: f32,
    velocity: f32,
    config: SpringConfig,
}

impl SpringChannel {
    fn new(config: SpringConfig) -> Self {
        Self {
            position: 0.0,
            velocity: 0.0,
            config: config.sanitized(),
        }
    }

    fn advance(&mut self, seconds: f64) {
        if seconds <= 0.0 {
            return;
        }

        let position = f64::from(self.position);
        let velocity = f64::from(self.velocity);
        let stiffness = f64::from(self.config.stiffness);
        let damping = f64::from(self.config.damping);
        let mass = f64::from(self.config.mass);
        let displacement = position - 1.0;
        let decay = damping / (2.0 * mass);
        let natural_frequency_squared = stiffness / mass;
        let discriminant = decay.mul_add(decay, -natural_frequency_squared);
        let tolerance = natural_frequency_squared.max(1.0) * 1.0e-10;

        let (next_displacement, next_velocity) = if discriminant < -tolerance {
            underdamped(displacement, velocity, decay, -discriminant, seconds)
        } else if discriminant > tolerance {
            overdamped(displacement, velocity, decay, discriminant, seconds)
        } else {
            critically_damped(displacement, velocity, decay, seconds)
        };

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Spring state is stored as f32 to match interpolation progress."
        )]
        {
            self.position = (next_displacement + 1.0) as f32;
            self.velocity = next_velocity as f32;
        }
    }

    fn is_settled(self) -> bool {
        (1.0 - self.position).abs() <= self.config.epsilon
            && self.velocity.abs() <= self.config.epsilon
    }

    fn reset_position(&mut self, position: f32) {
        self.position = position;
        self.velocity = 0.0;
    }
}

type Compositor<T> = Arc<dyn Fn(&[T]) -> T>;

/// An animation driven by one or more damped spring channels.
///
/// [`Spring::new`] creates one channel and preserves the original behavior in
/// which every field shares the same physical progress. Use
/// [`Spring::with_channels`] when fields need different stiffness, damping, or
/// mass. Each channel produces a complete `T`; the compositor selects the
/// fields owned by each channel.
///
/// Channels use the analytic solution of the damped harmonic oscillator. Time
/// is therefore not discarded for frame intervals greater than 100 ms, and
/// simulation results are stable across different frame subdivisions.
///
/// # Examples
///
/// ```
/// use aura_anim_core::{
///     Animation, Spring, SpringConfig,
///     timing::Duration,
/// };
///
/// let soft = SpringConfig {
///     stiffness: 80.0,
///     damping: 18.0,
///     ..SpringConfig::default()
/// };
/// let snappy = SpringConfig {
///     stiffness: 420.0,
///     damping: 28.0,
///     ..SpringConfig::default()
/// };
/// let mut spring = Spring::with_channels(
///     (0.0_f32, 0.0_f32),
///     (100.0, 1.0),
///     [soft, snappy],
///     |outputs| (outputs[0].0, outputs[1].1),
/// );
///
/// spring.tick(Duration::from_millis(100.0));
/// assert!(spring.value().1 > spring.value().0 / 100.0);
/// ```
#[derive(Clone)]
pub struct Spring<T: Animatable> {
    from: T,
    to: T,
    current: T,
    outputs: Vec<T>,
    channels: Vec<SpringChannel>,
    compose: Compositor<T>,
    state: AnimationState,
}

impl<T: Animatable + fmt::Debug> fmt::Debug for Spring<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Spring")
            .field("from", &self.from)
            .field("to", &self.to)
            .field("current", &self.current)
            .field("channels", &self.channels)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl<T: Animatable> Spring<T> {
    /// Creates a running spring animation with one physical channel.
    ///
    /// All fields in `T` share this channel. For independently configured
    /// fields, use [`Spring::with_channels`].
    #[must_use]
    pub fn new(from: T, to: T, config: SpringConfig) -> Self {
        Self::with_channels(from, to, [config], |outputs| outputs[0].clone())
    }

    /// Creates a running spring with independently configured channels.
    ///
    /// Every channel evaluates the full transition from `from` to `to` with
    /// its own physical state. `compose` must build the final value by
    /// selecting the fields owned by each channel, in the same order as
    /// `configs`.
    ///
    /// An empty configuration iterator falls back to one default channel.
    #[must_use]
    pub fn with_channels(
        from: T,
        to: T,
        configs: impl IntoIterator<Item = SpringConfig>,
        compose: impl Fn(&[T]) -> T + 'static,
    ) -> Self {
        let mut channels: Vec<_> = configs.into_iter().map(SpringChannel::new).collect();
        if channels.is_empty() {
            channels.push(SpringChannel::new(SpringConfig::default()));
        }
        let outputs = vec![from.clone(); channels.len()];

        Self {
            current: from.clone(),
            from,
            to,
            outputs,
            channels,
            compose: Arc::new(compose),
            state: AnimationState::Running,
        }
    }

    /// Returns the number of independently simulated physical channels.
    #[must_use]
    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    /// Restarts every channel from the current value toward `target`.
    ///
    /// Each channel keeps its normalized velocity so interrupted motion
    /// retains momentum while its position is rebased to the current value.
    pub fn retarget(&mut self, target: T) {
        self.from = self.current.clone();
        self.to = target;
        for (channel, output) in self.channels.iter_mut().zip(&mut self.outputs) {
            channel.position = 0.0;
            *output = self.from.clone();
        }
        self.state = AnimationState::Running;
    }

    fn sample(&mut self) {
        for (channel, output) in self.channels.iter().zip(&mut self.outputs) {
            *output = T::extrapolate(&self.from, &self.to, channel.position);
        }
        self.current = (self.compose)(&self.outputs);
    }

    fn integrate(&mut self, delta: Duration) {
        let seconds = delta.as_secs();
        for channel in &mut self.channels {
            channel.advance(seconds);
        }
        self.sample();

        if self.channels.iter().copied().all(SpringChannel::is_settled) {
            self.finish();
        }
    }
}

impl<T: Animatable> Animation<T> for Spring<T> {
    fn value(&self) -> &T {
        &self.current
    }

    fn state(&self) -> AnimationState {
        self.state
    }

    fn tick(&mut self, delta: Duration) {
        if self.state == AnimationState::Running {
            self.integrate(delta);
        }
    }

    fn pause(&mut self) {
        if self.state == AnimationState::Running {
            self.state = AnimationState::Paused;
        }
    }

    fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Running;
        }
    }

    fn cancel(&mut self) {
        if matches!(self.state, AnimationState::Running | AnimationState::Paused) {
            self.state = AnimationState::Canceled;
        }
    }

    fn seek(&mut self, progress: f32) {
        let progress = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        for channel in &mut self.channels {
            channel.reset_position(progress);
        }
        self.sample();
    }

    fn finish(&mut self) {
        for (channel, output) in self.channels.iter_mut().zip(&mut self.outputs) {
            channel.reset_position(1.0);
            *output = self.to.clone();
        }
        self.current = self.to.clone();
        self.state = AnimationState::Completed;
    }

    fn retarget(&mut self, target: &T) -> bool {
        self.retarget(target.clone());
        true
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn underdamped(
    displacement: f64,
    velocity: f64,
    decay: f64,
    negative_discriminant: f64,
    seconds: f64,
) -> (f64, f64) {
    let damped_frequency = negative_discriminant.sqrt();
    let angle = damped_frequency * seconds;
    let (sin, cos) = angle.sin_cos();
    let exponential = (-decay * seconds).exp();
    let sine_coefficient = (velocity + decay * displacement) / damped_frequency;
    let next_displacement = exponential * displacement.mul_add(cos, sine_coefficient * sin);
    let next_velocity = exponential
        * ((-decay * displacement + damped_frequency * sine_coefficient) * cos
            + (-decay * sine_coefficient - damped_frequency * displacement) * sin);

    (next_displacement, next_velocity)
}

fn critically_damped(displacement: f64, velocity: f64, decay: f64, seconds: f64) -> (f64, f64) {
    let linear_coefficient = velocity + decay * displacement;
    let exponential = (-decay * seconds).exp();
    let value = displacement + linear_coefficient * seconds;

    (
        value * exponential,
        (linear_coefficient - decay * value) * exponential,
    )
}

fn overdamped(
    displacement: f64,
    velocity: f64,
    decay: f64,
    discriminant: f64,
    seconds: f64,
) -> (f64, f64) {
    let root = discriminant.sqrt();
    let first_rate = -decay + root;
    let second_rate = -decay - root;
    let first_coefficient = (velocity - second_rate * displacement) / (first_rate - second_rate);
    let second_coefficient = displacement - first_coefficient;
    let first_term = first_coefficient * (first_rate * seconds).exp();
    let second_term = second_coefficient * (second_rate * seconds).exp();

    (
        first_term + second_term,
        first_rate * first_term + second_rate * second_term,
    )
}

#[cfg(test)]
mod tests {
    use super::{Spring, SpringConfig};
    use crate::{Animation, AnimationState, timing::Duration};
    use float_cmp::assert_approx_eq;

    #[test]
    fn channels_apply_independent_physics_to_selected_fields() {
        let slow = SpringConfig {
            stiffness: 40.0,
            damping: 14.0,
            ..SpringConfig::default()
        };
        let fast = SpringConfig {
            stiffness: 420.0,
            damping: 28.0,
            ..SpringConfig::default()
        };
        let mut spring = Spring::with_channels(
            (0.0_f32, 0.0_f32),
            (100.0, 100.0),
            [slow, fast],
            |outputs| (outputs[0].0, outputs[1].1),
        );

        spring.tick(Duration::from_millis(100.0));

        assert_eq!(spring.channel_count(), 2);
        assert!(spring.value().1 > spring.value().0 * 2.0);
    }

    #[test]
    fn analytic_solution_preserves_full_delta() {
        let config = SpringConfig::default();
        let mut single_tick = Spring::new(0.0_f32, 1.0, config);
        let mut divided_ticks = Spring::new(0.0_f32, 1.0, config);

        single_tick.tick(Duration::from_millis(500.0));
        for _ in 0..5 {
            divided_ticks.tick(Duration::from_millis(100.0));
        }

        assert_approx_eq!(
            f32,
            *single_tick.value(),
            *divided_ticks.value(),
            epsilon = 0.000_01
        );
        assert!(*single_tick.value() > 0.9);
    }

    #[test]
    fn completion_waits_for_every_channel() {
        let fast = SpringConfig {
            stiffness: 500.0,
            damping: 50.0,
            epsilon: 0.01,
            ..SpringConfig::default()
        };
        let slow = SpringConfig {
            stiffness: 20.0,
            damping: 8.0,
            epsilon: 0.000_1,
            ..SpringConfig::default()
        };
        let mut spring =
            Spring::with_channels((0.0_f32, 0.0_f32), (1.0, 1.0), [fast, slow], |outputs| {
                (outputs[0].0, outputs[1].1)
            });

        spring.tick(Duration::from_millis(500.0));

        assert_eq!(spring.state(), AnimationState::Running);
        assert!((1.0 - spring.value().0).abs() < (1.0 - spring.value().1).abs());
    }

    #[test]
    fn empty_channel_list_uses_default_channel() {
        let mut spring =
            Spring::with_channels(0.0_f32, 1.0, std::iter::empty(), |outputs| outputs[0]);

        spring.tick(Duration::from_millis(16.0));

        assert_eq!(spring.channel_count(), 1);
        assert!(*spring.value() > 0.0);
    }

    #[test]
    fn invalid_config_values_are_sanitized() {
        let mut spring = Spring::new(
            0.0_f32,
            1.0,
            SpringConfig {
                stiffness: f32::NAN,
                damping: f32::NEG_INFINITY,
                mass: 0.0,
                epsilon: f32::NAN,
            },
        );

        spring.tick(Duration::from_millis(16.0));

        assert!(spring.value().is_finite());
    }

    #[test]
    fn finish_sets_all_channels_to_the_exact_target() {
        let mut spring = Spring::with_channels(
            (0.0_f32, 0.0_f32),
            (10.0, 20.0),
            [SpringConfig::default(), SpringConfig::default()],
            |outputs| (outputs[0].0, outputs[1].1),
        );

        spring.finish();

        assert_eq!(spring.state(), AnimationState::Completed);
        assert_eq!(*spring.value(), (10.0, 20.0));
    }
}
