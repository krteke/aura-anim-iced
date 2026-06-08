use lilt::Easing;

use crate::traits::Animatable;

/// A value sample positioned at a time in a keyframe animation.
#[derive(Debug, Clone)]
pub struct Keyframe<T: Animatable> {
    time: f64,
    value: T,
    easing: Easing,
}

impl<T: Animatable> Keyframe<T> {
    /// Creates a keyframe at `time` milliseconds with default easing.
    #[must_use]
    pub fn new(time: f64, value: T) -> Self {
        let time = if time.is_finite() { time } else { 0.0 };

        Self {
            time,
            value,
            easing: Easing::default(),
        }
    }

    /// Returns the keyframe position in milliseconds.
    #[must_use]
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Returns the value stored by the keyframe.
    #[must_use]
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the easing applied after this keyframe.
    #[must_use]
    pub fn easing(&self) -> Easing {
        self.easing
    }

    /// Sets the easing applied after this keyframe.
    #[must_use]
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}
