use std::sync::Arc;

use crate::{
    interpolate_value,
    keyframes::Keyframes,
    property::{PropertyEntry, PropertySnapshot, PropertyValue, RawPropertySpec},
    timeline::Timeline,
    timing::{Duration, Timing},
};

/// Erased animation data owned by an internal runtime entry.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum AnimationSource {
    /// A keyframe track sampled directly by the runtime.
    Keyframes(Keyframes),
    /// A single-property transition sampled without keyframe track storage.
    PropertyTransition(PropertyTransitionSource),
    /// A timeline sampled by the runtime.
    Timeline(Arc<Timeline>),
}

impl From<Keyframes> for AnimationSource {
    fn from(value: Keyframes) -> Self {
        Self::Keyframes(value)
    }
}

impl From<Timeline> for AnimationSource {
    fn from(value: Timeline) -> Self {
        Self::Timeline(Arc::new(value))
    }
}

impl From<Arc<Timeline>> for AnimationSource {
    fn from(value: Arc<Timeline>) -> Self {
        Self::Timeline(value)
    }
}

impl From<PropertyTransitionSource> for AnimationSource {
    fn from(value: PropertyTransitionSource) -> Self {
        Self::PropertyTransition(value)
    }
}

impl AnimationSource {
    #[must_use]
    pub(crate) fn total_duration(&self) -> Option<Duration> {
        match self {
            Self::Keyframes(keyframes) => keyframes.timing().total_duration(),
            Self::PropertyTransition(transition) => transition.timing().total_duration(),
            Self::Timeline(timeline) => timeline.total_duration(),
        }
    }

    #[must_use]
    pub(crate) fn sample_at(&self, elapsed: impl Into<Duration>) -> Option<PropertySnapshot> {
        let elapsed = elapsed.into();

        match self {
            Self::Keyframes(keyframes) => sample_keyframes(keyframes, elapsed),
            Self::PropertyTransition(transition) => transition.sample_at(elapsed),
            Self::Timeline(timeline) => timeline.sample_at(elapsed),
        }
    }

    pub(crate) fn sample_into(
        &self,
        elapsed: impl Into<Duration>,
        output: &mut PropertySnapshot,
    ) -> bool {
        let elapsed = elapsed.into();

        match self {
            Self::Keyframes(keyframes) => sample_keyframes_into(keyframes, elapsed, output),
            Self::PropertyTransition(transition) => transition.sample_into(elapsed, output),
            Self::Timeline(timeline) => {
                if let Some(snapshot) = timeline.sample_at(elapsed) {
                    output.replace_from(&snapshot);
                    true
                } else {
                    output.clear();
                    false
                }
            }
        }
    }

    #[must_use]
    pub(crate) fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        match self {
            Self::Keyframes(keyframes) => keyframes.sample_completion(),
            Self::PropertyTransition(transition) => transition.completion_snapshot(),
            Self::Timeline(timeline) => timeline.completion_snapshot(),
        }
    }

    pub(crate) fn completion_snapshot_into(&self, output: &mut PropertySnapshot) -> bool {
        match self {
            Self::Keyframes(keyframes) => keyframes.sample_completion_into(output),
            Self::PropertyTransition(transition) => transition.completion_snapshot_into(output),
            Self::Timeline(timeline) => {
                if let Some(snapshot) = timeline.completion_snapshot() {
                    output.replace_from(&snapshot);
                    true
                } else {
                    output.clear();
                    false
                }
            }
        }
    }
}

/// Single-property transition source used by behavior transitions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PropertyTransitionSource {
    spec: RawPropertySpec,
    from: PropertyValue,
    to: PropertyValue,
    timing: Timing,
}

impl PropertyTransitionSource {
    pub(crate) const fn new(
        spec: RawPropertySpec,
        from: PropertyValue,
        to: PropertyValue,
        timing: Timing,
    ) -> Self {
        Self {
            spec,
            from,
            to,
            timing,
        }
    }

    const fn timing(&self) -> Timing {
        self.timing
    }

    fn sample_at(&self, elapsed: Duration) -> Option<PropertySnapshot> {
        let timing = self.timing.normalize_elapsed(elapsed.as_millis());

        if !timing.has_sample() {
            return None;
        }

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
        )]
        self.sample_progress(timing.iteration_progress as f32)
    }

    fn sample_into(&self, elapsed: Duration, output: &mut PropertySnapshot) -> bool {
        let timing = self.timing.normalize_elapsed(elapsed.as_millis());

        if !timing.has_sample() {
            output.clear();
            return false;
        }

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
        )]
        self.sample_progress_into(timing.iteration_progress as f32, output)
    }

    fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        let iteration_count = self.timing.iterations().finite_count()?;
        let offset = self.timing.direction().end_progress(iteration_count);

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
        )]
        self.sample_progress(offset as f32)
    }

    fn completion_snapshot_into(&self, output: &mut PropertySnapshot) -> bool {
        let Some(iteration_count) = self.timing.iterations().finite_count() else {
            output.clear();
            return false;
        };
        let offset = self.timing.direction().end_progress(iteration_count);

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
        )]
        self.sample_progress_into(offset as f32, output)
    }

    fn sample_progress(&self, progress: f32) -> Option<PropertySnapshot> {
        let progress = self.timing.easing().value(progress);
        let value = interpolate_value(self.from, self.to, progress)?;
        let mut snapshot = PropertySnapshot::with_capacity(1);

        snapshot.push(PropertyEntry::new_unchecked(self.spec, value));

        Some(snapshot)
    }

    fn sample_progress_into(&self, progress: f32, output: &mut PropertySnapshot) -> bool {
        let progress = self.timing.easing().value(progress);
        let Some(value) = interpolate_value(self.from, self.to, progress) else {
            output.clear();
            return false;
        };

        output.clear();
        output.push(PropertyEntry::new_unchecked(self.spec, value));

        true
    }
}

fn sample_keyframes(keyframes: &Keyframes, elapsed: Duration) -> Option<PropertySnapshot> {
    let timing = keyframes.timing().normalize_elapsed(elapsed.as_millis());

    if !timing.has_sample() {
        return None;
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
    )]
    keyframes.sample_at(timing.iteration_progress as f32)
}

fn sample_keyframes_into(
    keyframes: &Keyframes,
    elapsed: Duration,
    output: &mut PropertySnapshot,
) -> bool {
    let timing = keyframes.timing().normalize_elapsed(elapsed.as_millis());

    if !timing.has_sample() {
        output.clear();
        return false;
    }

    #[allow(
        clippy::cast_possible_truncation,
        reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
    )]
    keyframes.sample_into(timing.iteration_progress as f32, output)
}
