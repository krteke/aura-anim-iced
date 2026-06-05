use crate::{timing::Timing, traits::Animatable, tween::Tween};

pub struct TweenBuilder<T: Animatable> {
    start: T,
    end: T,
    timing: Timing,
}

impl<T: Animatable> TweenBuilder<T> {
    pub fn new(start: T, end: T) -> Self {
        Self {
            start,
            end,
            timing: Timing::default(),
        }
    }

    pub fn start(&self) -> &T {
        &self.start
    }

    pub fn end(&self) -> &T {
        &self.end
    }

    pub fn with_timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }

    pub fn build(self) -> Tween<T> {
        Tween::from_builder(self.start, self.end, self.timing)
    }
}
