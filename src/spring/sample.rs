/// A sampled scalar spring value and velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScalarSpringSample {
    value: f32,
    velocity: f32,
}

impl ScalarSpringSample {
    /// Creates a scalar spring sample.
    #[must_use]
    pub const fn new(value: f32, velocity: f32) -> Self {
        Self { value, velocity }
    }

    /// Returns the sampled value.
    #[must_use]
    pub const fn value(self) -> f32 {
        self.value
    }

    /// Returns the sampled velocity in value units per second.
    #[must_use]
    pub const fn velocity(self) -> f32 {
        self.velocity
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SpringDisplacementSample {
    displacement: f64,
    velocity: f64,
}

impl SpringDisplacementSample {
    pub(super) fn displacement(&self) -> f64 {
        self.displacement
    }

    pub(super) fn velocity(&self) -> f64 {
        self.velocity
    }
}

pub(super) fn sample_underdamped(
    displacement: f64,
    velocity: f64,
    elapsed_secs: f64,
    omega: f64,
    damping_ratio: f64,
) -> SpringDisplacementSample {
    let damped_omega = omega * (1.0 - damping_ratio.powi(2)).sqrt();
    let envelope = (-damping_ratio * omega * elapsed_secs).exp();
    let phase_cos = (damped_omega * elapsed_secs).cos();
    let phase_sin = (damped_omega * elapsed_secs).sin();
    let coefficient = (velocity + damping_ratio * omega * displacement) / damped_omega;
    let position_term = displacement * phase_cos + coefficient * phase_sin;
    let velocity_term = -damping_ratio * omega * position_term
        + (-displacement * damped_omega * phase_sin + coefficient * damped_omega * phase_cos);

    SpringDisplacementSample {
        displacement: envelope * position_term,
        velocity: envelope * velocity_term,
    }
}

pub(super) fn sample_critical(
    displacement: f64,
    velocity: f64,
    elapsed_secs: f64,
    omega: f64,
) -> SpringDisplacementSample {
    let envelope = (-omega * elapsed_secs).exp();
    let coefficient = velocity + omega * displacement;
    let position_term = displacement + coefficient * elapsed_secs;

    SpringDisplacementSample {
        displacement: envelope * position_term,
        velocity: envelope * (coefficient - omega * position_term),
    }
}

pub(super) fn sample_overdamped(
    displacement: f64,
    velocity: f64,
    elapsed_secs: f64,
    omega: f64,
    damping_ratio: f64,
) -> SpringDisplacementSample {
    let root = (damping_ratio.powi(2) - 1.0).sqrt();
    let slow_root = -omega * (damping_ratio - root);
    let fast_root = -omega * (damping_ratio + root);
    let slow_coefficient = (velocity - fast_root * displacement) / (slow_root - fast_root);
    let fast_coefficient = displacement - slow_coefficient;
    let slow = (slow_root * elapsed_secs).exp();
    let fast = (fast_root * elapsed_secs).exp();

    SpringDisplacementSample {
        displacement: slow_coefficient * slow + fast_coefficient * fast,
        velocity: slow_coefficient * slow_root * slow + fast_coefficient * fast_root * fast,
    }
}
