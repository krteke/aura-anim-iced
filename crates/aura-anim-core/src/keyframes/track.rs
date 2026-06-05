use lilt::Easing;
use std::cmp::Ordering;

use crate::{
    handle::AnimationHandle,
    keyframes::keyframe::Keyframe,
    timing::IterationCount,
    traits::{Animatable, Playable, Update},
};

#[derive(Debug, Clone, Copy)]
struct KeyframeTrackStatus {
    elapsed: f64,
    completed: bool,
    iter_count: u32,
}

impl KeyframeTrackStatus {
    fn init() -> Self {
        Self {
            elapsed: 0.0,
            completed: false,
            iter_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyframeTrack<T: Animatable> {
    id: AnimationHandle,
    frames: Vec<Keyframe<T>>,
    iterations: IterationCount,

    status: KeyframeTrackStatus,
}

impl<T: Animatable> Default for KeyframeTrack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Animatable> KeyframeTrack<T> {
    pub fn new() -> Self {
        Self {
            id: AnimationHandle::new(),
            frames: Vec::new(),
            iterations: IterationCount::ONCE,
            status: KeyframeTrackStatus::init(),
        }
    }

    pub fn id(&self) -> AnimationHandle {
        self.id
    }

    pub fn frames(&self) -> &[Keyframe<T>] {
        &self.frames
    }

    pub fn iterations(&self) -> IterationCount {
        self.iterations
    }

    pub fn with_iterations(mut self, iterations: IterationCount) -> Self {
        self.iterations = iterations;
        self
    }

    pub fn push(self, time: f64, value: T) -> Self {
        self.push_eased(time, value, Easing::Linear)
    }

    pub fn push_eased(mut self, time: f64, value: T, easing: Easing) -> Self {
        let frame = Keyframe::new(time, value).with_easing(easing);
        self.push_frame(frame)
    }

    pub fn push_frame(mut self, frame: Keyframe<T>) -> Self {
        match self.frames.binary_search_by(|existing| {
            existing
                .time()
                .partial_cmp(&frame.time())
                .unwrap_or(Ordering::Equal)
        }) {
            Ok(index) => self.frames[index] = frame,
            Err(index) => self.frames.insert(index, frame),
        }
        self
    }

    pub fn is_complete(&self) -> bool {
        self.frames.len() < 2 || self.status.completed
    }

    pub fn duration(&self) -> f64 {
        self.frames.last().map_or(0.0, |frame| frame.time())
    }

    pub fn progress(&self) -> f64 {
        let Some(total) = self.playback_duration() else {
            let base = self.duration();
            return (self.active_time() / base).clamp(0.0, 1.0);
        };

        if total <= 0.0 {
            return 1.0;
        }

        (self.status.elapsed / total).clamp(0.0, 1.0)
    }

    pub fn value_at(&self, at: f64) -> Option<T> {
        if self.frames.is_empty() {
            return None;
        }
        if self.frames.len() == 1 {
            return Some(self.frames[0].value().clone());
        }

        let duration = self.duration();
        let t = at.clamp(0.0, duration);

        if t <= self.frames[0].time() {
            return Some(self.frames[0].value().clone());
        }
        let upper = self.frames.partition_point(|frame| frame.time() <= t);
        let index = upper.saturating_sub(1);
        let current = &self.frames[index];
        let next = &self.frames[index + 1];
        let span = (next.time() - current.time()).max(f64::EPSILON);
        let local_t = ((t - current.time()) / span).clamp(0.0, 1.0);
        let curve_t = current.easing().value(local_t as f32);
        Some(current.value().lerp(next.value(), curve_t))
    }

    pub fn value(&self) -> Option<T> {
        self.value_at(self.active_time())
    }

    fn active_time(&self) -> f64 {
        let base = self.duration();
        if base <= 0.0 {
            return 0.0;
        }

        match self.iterations.finite_count() {
            Some(1) => self.status.elapsed.min(base),
            Some(_) => {
                if self.status.completed {
                    base
                } else {
                    self.status.elapsed % base
                }
            }
            None => self.status.elapsed % base,
        }
    }

    fn playback_duration(&self) -> Option<f64> {
        let base = self.duration();
        if base <= 0.0 {
            return Some(0.0);
        }

        match self.iterations.finite_count() {
            Some(count) => Some(base * count as f64),
            None => Some(base),
        }
    }
}

impl<T: Animatable> Update for KeyframeTrack<T> {
    fn update(&mut self, dt: f64) -> bool {
        if self.is_complete() {
            return false;
        }

        let base = self.duration();
        if base <= 0.0 {
            self.status.completed = true;
            return false;
        }

        self.status.elapsed += dt;
        self.status.iter_count = (self.status.elapsed / base).floor() as u32;

        if let Some(count) = self.iterations.finite_count()
            && let total = base * count as f64
            && self.status.elapsed >= total
        {
            self.status.elapsed = total;
            self.status.iter_count = count;
            self.status.completed = true;
            return false;
        }

        true
    }
}

impl<T: Animatable> Playable for KeyframeTrack<T> {
    fn duration(&self) -> f32 {
        todo!()
    }

    fn is_complete(&self) -> bool {
        todo!()
    }

    fn seek(&mut self, progress: f32) {
        todo!()
    }
}
