use lilt::Easing;

use crate::{timing::Delay, traits::Animatable};

#[derive(Debug, Clone)]
pub struct Keyframe<T: Animatable> {
    // milliseconds
    time: f64,
    value: T,
    easing: Easing,
    delay: Delay,
}

impl<T: Animatable> Keyframe<T> {
    pub fn new(time: f64, value: T) -> Self {
        Self {
            time,
            value,
            easing: Easing::default(),
            delay: Delay::default(),
        }
    }

    pub fn time(&self) -> f64 {
        self.time
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn easing(&self) -> Easing {
        self.easing
    }

    pub fn delay(&self) -> Delay {
        self.delay
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn with_delay(mut self, delay: Delay) -> Self {
        self.delay = delay;
        self
    }
}
