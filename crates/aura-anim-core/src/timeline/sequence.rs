use crate::{Animatable, Animation, AnimationState, timeline::normalized, timing::Duration};

pub struct Sequence<T: Animatable> {
    children: Vec<Box<dyn Animation<T>>>,
    current: T,
    index: usize,
    state: AnimationState,
}

impl<T: Animatable> Sequence<T> {
    pub fn new(initial: T) -> Self {
        Self {
            children: Vec::new(),
            current: initial,
            index: 0,
            state: AnimationState::Idle,
        }
    }

    pub fn then(mut self, animation: impl Animation<T>) -> Self {
        self.push(animation);
        self
    }

    pub fn push(&mut self, animation: impl Animation<T>) {
        self.children.push(Box::new(animation));
        self.state = AnimationState::Running;
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

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
            let scaled = progress * self.children.len() as f32;
            let index = (scaled.floor() as usize).min(self.children.len() - 1);
            for child in &mut self.children[..index] {
                child.finish();
            }
            self.index = index;
            self.children[index].seek((scaled - index as f32).clamp(0.0, 1.0));
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
