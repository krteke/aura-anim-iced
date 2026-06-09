use crate::{Animatable, Animation, AnimationState, timeline::normalized, timing::Duration};

type Compositor<T> = Box<dyn Fn(&[T]) -> T>;

/// Runs child animations together and composes their output values.
pub struct Parallel<T: Animatable> {
    children: Vec<Box<dyn Animation<T>>>,
    outputs: Vec<T>,
    current: T,
    compose: Compositor<T>,
    state: AnimationState,
}

impl<T: Animatable> Parallel<T> {
    /// Creates an empty parallel animation with an output compositor.
    #[must_use]
    pub fn new(initial: T, compose: impl Fn(&[T]) -> T + 'static) -> Self {
        Self {
            children: Vec::new(),
            outputs: Vec::new(),
            current: initial,
            compose: Box::new(compose),
            state: AnimationState::Idle,
        }
    }

    /// Appends an animation and returns the updated parallel composition.
    #[must_use]
    pub fn with(mut self, animation: impl Animation<T>) -> Self {
        self.push(animation);
        self
    }

    /// Appends an animation to the parallel composition.
    pub fn push(&mut self, animation: impl Animation<T>) {
        self.outputs.push(animation.value().clone());
        self.children.push(Box::new(animation));
        self.state = AnimationState::Running;
    }

    /// Returns the number of child animations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.children.len()
    }

    /// Returns whether the composition contains no child animations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    fn compose_outputs(&mut self) {
        if !self.outputs.is_empty() {
            self.current = (self.compose)(&self.outputs);
        }
    }
}

impl<T: Animatable> Animation<T> for Parallel<T> {
    fn value(&self) -> &T {
        &self.current
    }

    fn state(&self) -> AnimationState {
        self.state
    }

    fn duration(&self) -> Option<Duration> {
        self.children
            .iter()
            .map(Animation::duration)
            .try_fold(Duration::ZERO, |longest, duration| {
                duration.map(|duration| longest.max(duration))
            })
    }

    fn tick(&mut self, delta: Duration) {
        self.advance(delta);
    }

    fn advance(&mut self, delta: Duration) -> Duration {
        if self.state != AnimationState::Running {
            return delta;
        }
        if self.children.is_empty() {
            self.state = AnimationState::Completed;
            return delta;
        }

        let mut overflow = delta;
        let mut completed = true;
        for (index, child) in self.children.iter_mut().enumerate() {
            let child_overflow = child.advance(delta);
            overflow = overflow.min(child_overflow);
            self.outputs[index] = child.value().clone();
            completed &= child.state() == AnimationState::Completed;
        }
        self.compose_outputs();

        if completed {
            self.state = AnimationState::Completed;
            overflow
        } else {
            Duration::ZERO
        }
    }

    fn pause(&mut self) {
        if self.state == AnimationState::Running {
            for child in &mut self.children {
                child.pause();
            }
            self.state = AnimationState::Paused;
        }
    }

    fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            for child in &mut self.children {
                child.resume();
            }
            self.state = AnimationState::Running;
        }
    }

    fn cancel(&mut self) {
        if matches!(self.state, AnimationState::Running | AnimationState::Paused) {
            for child in &mut self.children {
                child.cancel();
            }
            self.state = AnimationState::Canceled;
        }
    }

    fn seek(&mut self, progress: f32) {
        let progress = normalized(progress);
        let duration = self.duration();
        for (index, child) in self.children.iter_mut().enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let child_progress = match (duration, child.duration()) {
                (Some(total), Some(child_duration)) if !child_duration.is_zero() => {
                    (total.as_secs() * f64::from(progress) / child_duration.as_secs())
                        .clamp(0.0, 1.0) as f32
                }
                (Some(_), Some(_)) => 1.0,
                _ => progress,
            };
            child.seek(child_progress);
            self.outputs[index] = child.value().clone();
        }
        self.compose_outputs();
        self.state = if progress >= 1.0 {
            AnimationState::Completed
        } else {
            AnimationState::Running
        };
    }

    fn finish(&mut self) {
        for (index, child) in self.children.iter_mut().enumerate() {
            child.finish();
            self.outputs[index] = child.value().clone();
        }
        self.compose_outputs();
        self.state = AnimationState::Completed;
    }
}

#[cfg(test)]
mod tests {
    use super::Parallel;
    use crate::{Animation, AnimationState, Tween, timing::Timing};
    use float_cmp::assert_approx_eq;

    #[test]
    fn empty_parallel_remains_idle_when_advanced() {
        let mut parallel = Parallel::new(2.0_f32, |values| values[0]);

        let overflow = parallel.advance(crate::timing::Duration::from_millis(10.0));

        assert_eq!(parallel.state(), AnimationState::Idle);
        assert_eq!(overflow, crate::timing::Duration::from_millis(10.0));
        assert_approx_eq!(f32, *parallel.value(), 2.0);
    }

    #[test]
    fn seek_scales_progress_by_child_duration() {
        let mut parallel = Parallel::new(0.0_f32, |values| values.iter().sum())
            .with(Tween::between(0.0, 10.0, Timing::new(100.0)))
            .with(Tween::between(0.0, 20.0, Timing::new(200.0)));

        parallel.seek(0.5);

        assert_eq!(parallel.state(), AnimationState::Running);
        assert_approx_eq!(f32, *parallel.value(), 20.0);
    }

    #[test]
    fn cancel_propagates_to_children() {
        let mut parallel = Parallel::new(0.0_f32, |values| values.iter().sum())
            .with(Tween::between(0.0, 10.0, Timing::new(100.0)))
            .with(Tween::between(0.0, 20.0, Timing::new(200.0)));

        parallel.cancel();
        parallel.tick(crate::timing::Duration::from_millis(100.0));

        assert_eq!(parallel.state(), AnimationState::Canceled);
        assert_approx_eq!(f32, *parallel.value(), 0.0);
    }
}
