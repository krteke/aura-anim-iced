use crate::{
    handle::AnimationHandle,
    timing::{Direction, Timing},
    traits::{Animatable, Playable, Update},
    tween::builder::TweenBuilder,
};

mod builder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenState {
    Idle,
    Running,
    Paused,
    Completed,
}

#[derive(Clone, Debug, Copy)]
struct TweenStatus {
    elapsed: f64,
    state: TweenState,
    iterations: u32,
    reverse: bool,
}

impl TweenStatus {
    fn init(reverse: bool) -> Self {
        Self {
            elapsed: 0.0,
            state: TweenState::Idle,
            iterations: 0,
            reverse,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tween<T: Animatable> {
    id: AnimationHandle,
    start: T,
    end: T,
    timing: Timing,
    status: TweenStatus,
}

impl<T: Animatable> Tween<T> {
    pub fn new(start: T, end: T) -> TweenBuilder<T> {
        TweenBuilder::new(start, end)
    }

    pub fn id(&self) -> AnimationHandle {
        self.id
    }

    pub fn start(&self) -> &T {
        &self.start
    }

    pub fn end(&self) -> &T {
        &self.end
    }

    pub fn timing(&self) -> &Timing {
        &self.timing
    }

    pub fn with_timing(mut self, timing: Timing) -> Self {
        self.timing = timing;
        self
    }

    pub(crate) fn from_builder(start: T, end: T, timing: Timing) -> Self {
        let reverse = match timing.direction() {
            Direction::Normal | Direction::Alternate => false,
            Direction::Reverse | Direction::AlternateReverse => true,
        };

        Self {
            id: AnimationHandle::new(),
            start,
            end,
            timing,
            status: TweenStatus::init(reverse),
        }
    }
}

impl<T: Animatable> Update for Tween<T> {
    fn update(&mut self, dt: f64) -> bool {
        if self.timing.duration().is_zero() {
            self.status.state = TweenState::Completed;
            return false;
        }

        let dt = dt.max(0.0);

        match self.status.state {
            TweenState::Completed => return false,
            TweenState::Paused => return true,
            TweenState::Idle => {
                self.status.elapsed += dt;
                if self.status.elapsed < self.timing.delay().as_millis() {
                    return true;
                }
                let overflow = self.status.elapsed - self.timing.delay().as_millis();
                self.status.state = TweenState::Running;
                self.status.elapsed += overflow * self.timing.playback_rate();
            }
            TweenState::Running => {
                self.status.elapsed += dt * self.timing.playback_rate();
            }
        }

        while self.status.elapsed >= self.timing.duration().as_millis() {
            match self.timing.iterations().finite_count() {
                Some(1) => {
                    self.status.elapsed = self.timing.duration().as_millis();
                    self.status.state = TweenState::Completed;
                    self.status.iterations += 1;
                    return false;
                }
                Some(i) => {
                    self.status.iterations += 1;
                    if self.status.iterations >= i {
                        self.status.elapsed = self.timing.duration().as_millis();
                        self.status.state = TweenState::Completed;
                        return false;
                    }
                    self.status.elapsed -= self.timing.duration().as_millis();
                    self.status.reverse = self
                        .timing
                        .direction()
                        .is_reversed_iteration(self.status.iterations);
                }
                None => {
                    self.status.elapsed -= self.timing.duration().as_millis();
                    self.status.reverse = self
                        .timing
                        .direction()
                        .is_reversed_iteration(self.status.iterations);
                }
            }
        }

        true
    }
}

impl<T: Animatable> Playable for Tween<T> {
    fn duration(&self) -> f32 {
        todo!()
    }

    fn seek(&mut self, progress: f32) {
        todo!()
    }

    fn is_complete(&self) -> bool {
        todo!()
    }
}
