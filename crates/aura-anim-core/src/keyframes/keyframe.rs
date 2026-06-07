use lilt::Easing;

use crate::traits::Animatable;

#[derive(Debug, Clone)]
pub struct Keyframe<T: Animatable> {
    time: f64,
    value: T,
    easing: Easing,
}

impl<T: Animatable> Keyframe<T> {
    pub fn new(time: f64, value: T) -> Self {
        let time = if time.is_finite() { time } else { 0.0 };

        Self {
            time,
            value,
            easing: Easing::default(),
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

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}
