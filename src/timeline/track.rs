use crate::{keyframes::Keyframes, property::PropertySnapshot, timing::Duration};

/// A keyframe track placed in a timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    name: Option<String>,
    keyframes: Keyframes,
}

impl Track {
    /// Creates a track from keyframes.
    #[must_use]
    pub const fn new(keyframes: Keyframes) -> Self {
        Self {
            name: None,
            keyframes,
        }
    }

    /// Sets the track name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Returns the track name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the track keyframes.
    #[must_use]
    pub const fn keyframes(&self) -> &Keyframes {
        &self.keyframes
    }

    /// Returns the finite total duration of the track, or `None` for infinite timing.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        self.keyframes.timing().total_duration()
    }

    /// Samples this track at local timeline `offset`.
    #[must_use]
    pub fn sample_at(&self, offset: impl Into<Duration>) -> Option<PropertySnapshot> {
        let timing = self
            .keyframes
            .timing()
            .normalize_elapsed(offset.into().as_millis());

        if !timing.has_sample() {
            return None;
        }

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
        )]
        self.keyframes.sample_at(timing.iteration_progress as f32)
    }
}
