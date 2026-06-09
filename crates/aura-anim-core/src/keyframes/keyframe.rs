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

    pub(crate) fn set_rate(&mut self, rate: f64) {
        if rate.is_finite() && rate > 0.0 {
            let scaled = self.time / rate;
            self.time = if scaled.is_finite() { scaled } else { f64::MAX };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Keyframe;
    use float_cmp::assert_approx_eq;
    use lilt::Easing;

    #[test]
    fn invalid_time_is_replaced_with_zero() {
        assert_approx_eq!(f64, Keyframe::new(f64::NAN, 1.0_f32).time(), 0.0);
        assert_approx_eq!(f64, Keyframe::new(f64::INFINITY, 1.0_f32).time(), 0.0);
    }

    #[test]
    fn accessors_return_stored_values() {
        let frame = Keyframe::new(25.0, 3_i32).with_easing(Easing::EaseIn);

        assert_approx_eq!(f64, frame.time(), 25.0);
        assert_eq!(*frame.value(), 3);
        assert_eq!(frame.easing(), Easing::EaseIn);
    }
}
