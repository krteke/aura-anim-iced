use crate::{Animatable, Animation, AnimationState, timeline::normalized, timing::Duration};

/// Runs child animations one after another.
///
/// # Examples
///
/// ```
/// use aura_anim_core::{
///     Animation, Sequence, Tween,
///     timing::{Duration, Timing},
/// };
///
/// let mut sequence = Sequence::new(0.0_f32)
///     .then(Tween::between(0.0, 1.0, Timing::new(100.0)))
///     .then(Tween::between(1.0, 2.0, Timing::new(100.0)));
///
/// sequence.tick(Duration::from_millis(150.0));
/// assert_eq!(*sequence.value(), 1.5);
/// ```
pub struct Sequence<T: Animatable> {
    children: Vec<Box<dyn Animation<T>>>,
    current: T,
    index: usize,
    state: AnimationState,
}

impl<T: Animatable> Sequence<T> {
    /// Creates an empty sequence with an initial output value.
    #[must_use]
    pub fn new(initial: T) -> Self {
        Self {
            children: Vec::new(),
            current: initial,
            index: 0,
            state: AnimationState::Idle,
        }
    }

    /// Appends an animation and returns the updated sequence.
    #[must_use]
    pub fn then(mut self, animation: impl Animation<T>) -> Self {
        self.push(animation);
        self
    }

    /// Appends an animation to the sequence.
    pub fn push(&mut self, animation: impl Animation<T>) {
        self.children.push(Box::new(animation));
        self.state = AnimationState::Running;
    }

    /// Returns the number of child animations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns whether the sequence contains no child animations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    fn sync_current(&mut self) {
        if let Some(child) = self.children.get(self.index) {
            self.current = child.value().clone();
        }
    }
}

impl<T: Animatable> Animation<T> for Sequence<T> {
    fn value(&self) -> &T {
        &self.current
    }

    fn state(&self) -> AnimationState {
        self.state
    }

    fn duration(&self) -> Option<Duration> {
        self.children
            .iter()
            .try_fold(Duration::ZERO, |total, child| {
                child.duration().map(|duration| total + duration)
            })
    }

    fn tick(&mut self, delta: Duration) {
        self.advance(delta);
    }

    fn advance(&mut self, delta: Duration) -> Duration {
        if self.state != AnimationState::Running {
            return delta;
        }

        let mut remaining = delta;
        loop {
            let Some(child) = self.children.get_mut(self.index) else {
                self.state = AnimationState::Completed;
                return remaining;
            };

            remaining = child.advance(remaining);
            self.current = child.value().clone();
            if child.state() != AnimationState::Completed {
                return Duration::ZERO;
            }

            self.index += 1;
            if self.index >= self.children.len() {
                self.state = AnimationState::Completed;
                return remaining;
            }
            if remaining.is_zero() {
                return Duration::ZERO;
            }
        }
    }

    fn pause(&mut self) {
        if self.state == AnimationState::Running {
            if let Some(child) = self.children.get_mut(self.index) {
                child.pause();
            }
            self.state = AnimationState::Paused;
        }
    }

    fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            if let Some(child) = self.children.get_mut(self.index) {
                child.resume();
            }
            self.state = AnimationState::Running;
        }
    }

    fn cancel(&mut self) {
        if matches!(self.state, AnimationState::Running | AnimationState::Paused) {
            if let Some(child) = self.children.get_mut(self.index) {
                child.cancel();
            }
            self.state = AnimationState::Canceled;
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    fn seek(&mut self, progress: f32) {
        if self.children.is_empty() {
            self.state = AnimationState::Completed;
            return;
        }

        let progress = normalized(progress);
        if let Some(total) = self.duration() {
            let mut target = total.as_secs() * f64::from(progress);
            for (index, child) in self.children.iter_mut().enumerate() {
                let duration = child.duration().unwrap_or(Duration::ZERO).as_secs();
                if target >= duration {
                    child.finish();
                    target -= duration;
                    self.index = index + 1;
                } else {
                    let local = if duration <= 0.0 {
                        1.0
                    } else {
                        (target / duration) as f32
                    };
                    child.seek(local);
                    self.index = index;
                    break;
                }
            }
        } else {
            let len = self.children.len() as f32;
            let scaled = progress * len;
            let index = (scaled.floor()).min(len - 1.0);
            let index_usize = index as usize;

            for child in &mut self.children[..index_usize] {
                child.finish();
            }
            self.index = index_usize;
            self.children[index_usize].seek((scaled - index).clamp(0.0, 1.0));
        }

        if self.index >= self.children.len() {
            self.index = self.children.len() - 1;
        }
        self.sync_current();
        self.state = if progress >= 1.0 {
            AnimationState::Completed
        } else {
            AnimationState::Running
        };
    }

    fn finish(&mut self) {
        for child in &mut self.children {
            child.finish();
        }
        if let Some(child) = self.children.last() {
            self.current = child.value().clone();
        }
        self.index = self.children.len();
        self.state = AnimationState::Completed;
    }
}

#[cfg(test)]
mod tests {
    use super::Sequence;
    use crate::{Animation, AnimationState, Tween, timing::Timing};
    use float_cmp::assert_approx_eq;

    #[test]
    fn empty_sequence_remains_idle_when_advanced() {
        let mut sequence = Sequence::new(2.0_f32);

        let overflow = sequence.advance(crate::timing::Duration::from_millis(10.0));

        assert_eq!(sequence.state(), AnimationState::Idle);
        assert_eq!(overflow, crate::timing::Duration::from_millis(10.0));
        assert_approx_eq!(f32, *sequence.value(), 2.0);
    }

    #[test]
    fn seek_updates_current_child_value() {
        let mut sequence = Sequence::new(0.0_f32)
            .then(Tween::between(0.0, 10.0, Timing::new(100.0)))
            .then(Tween::between(10.0, 20.0, Timing::new(100.0)));

        sequence.seek(0.75);

        assert_eq!(sequence.state(), AnimationState::Running);
        assert_approx_eq!(f32, *sequence.value(), 15.0);
    }

    #[test]
    fn finish_uses_last_child_value() {
        let mut sequence = Sequence::new(0.0_f32)
            .then(Tween::between(0.0, 10.0, Timing::new(100.0)))
            .then(Tween::between(10.0, 20.0, Timing::new(100.0)));

        sequence.finish();

        assert_eq!(sequence.state(), AnimationState::Completed);
        assert_approx_eq!(f32, *sequence.value(), 20.0);
    }
}
