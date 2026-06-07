use crate::{
    timing::Duration,
    traits::{Animatable, Animation, AnimationState},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringConfig {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
    pub epsilon: f32,
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

#[derive(Debug, Clone)]
pub struct Spring<T: Animatable> {
    from: T,
    to: T,
    current: T,
    position: f32,
    velocity: f32,
    config: SpringConfig,
    state: AnimationState,
}

impl<T: Animatable> Spring<T> {
    pub fn new(from: T, to: T, config: SpringConfig) -> Self {
        Self {
            current: from.clone(),
            from,
            to,
            position: 0.0,
            velocity: 0.0,
            config,
            state: AnimationState::Running,
        }
    }

    pub fn retarget(&mut self, target: T) {
        self.from = self.current.clone();
        self.to = target;
        self.position = 0.0;
        self.state = AnimationState::Running;
    }

    fn integrate(&mut self, delta: Duration) {
        let mut remaining = delta.as_secs().min(0.1) as f32;
        let step = 1.0 / 120.0;
        let mass = self.config.mass.max(f32::EPSILON);

        while remaining > 0.0 {
            let dt = remaining.min(step);
            let acceleration = (self.config.stiffness * (1.0 - self.position)
                - self.config.damping * self.velocity)
                / mass;
            self.velocity += acceleration * dt;
            self.position += self.velocity * dt;
            remaining -= dt;
        }

        self.current = T::extrapolate(&self.from, &self.to, self.position);
        if (1.0 - self.position).abs() <= self.config.epsilon
            && self.velocity.abs() <= self.config.epsilon
        {
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
        self.position = if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        };
        self.velocity = 0.0;
        self.current = T::extrapolate(&self.from, &self.to, self.position);
    }

    fn finish(&mut self) {
        self.position = 1.0;
        self.velocity = 0.0;
        self.current = self.to.clone();
        self.state = AnimationState::Completed;
    }

    fn retarget(&mut self, target: &T) -> bool {
        self.retarget(target.clone());
        true
    }
}
