use crate::{
    keyframes::Keyframes,
    property::PropertySnapshot,
    timeline::Timeline,
    timing::{Duration, TimingPhase},
};

/// Animation data owned by a runtime entry.
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationSource {
    /// A keyframe track sampled directly by the runtime.
    Keyframes(Keyframes),
    /// A timeline sampled by the runtime.
    Timeline(Timeline),
}

impl From<Keyframes> for AnimationSource {
    fn from(value: Keyframes) -> Self {
        Self::Keyframes(value)
    }
}

impl From<Timeline> for AnimationSource {
    fn from(value: Timeline) -> Self {
        Self::Timeline(value)
    }
}

impl AnimationSource {
    /// Returns the finite total duration of this source, or `None` when infinite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        match self {
            Self::Keyframes(keyframes) => keyframes.timing().total_duration(),
            Self::Timeline(timeline) => timeline.total_duration(),
        }
    }

    /// Samples this source at elapsed runtime time.
    #[must_use]
    pub fn sample_at(&self, elapsed: impl Into<Duration>) -> Option<PropertySnapshot> {
        match self {
            Self::Keyframes(keyframes) => sample_keyframes(keyframes, elapsed.into()),
            Self::Timeline(timeline) => timeline.sample_at(elapsed),
        }
    }

    /// Returns the final visual state for this source.
    #[must_use]
    pub fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        match self {
            Self::Keyframes(keyframes) => keyframes.sample_at(1.0),
            Self::Timeline(timeline) => timeline.completion_snapshot(),
        }
    }
}

fn sample_keyframes(keyframes: &Keyframes, elapsed: Duration) -> Option<PropertySnapshot> {
    let timing = keyframes.timing().normalize_elapsed(elapsed.as_millis());

    if !timing.has_sample() {
        return None;
    }

    if timing.phase == TimingPhase::AfterEnd {
        return keyframes.sample_at(1.0);
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
    )]
    keyframes.sample_at(timing.iteration_progress as f32)
}
