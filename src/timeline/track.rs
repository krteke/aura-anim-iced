use crate::{
    keyframes::Keyframes,
    property::{PropertySnapshot, PropertyValue, UiProperty},
    timing::{Duration, Easing, Timing},
};

/// A keyframe track placed in a timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    name: Option<String>,
    keyframes: Keyframes,
}

/// A builder for creating property tracks.
pub struct PropertyTrackBuilder {
    property: UiProperty,
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

    /// Creates a track with an initial value for `property` at offset `0.0`.
    #[must_use]
    pub fn from(property: UiProperty, value: impl Into<PropertyValue>) -> PropertyTrackBuilder {
        PropertyTrackBuilder {
            property,
            keyframes: Keyframes::new().at(0.0, [(property, value.into())]),
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

    /// Inserts a keyframe at the end of the track with the given property and value.
    #[must_use]
    pub fn keyframe_at_end(
        mut self,
        property: UiProperty,
        value: impl Into<PropertyValue>,
    ) -> Self {
        self.keyframes = self.keyframes.at(1.0, [(property, value.into())]);
        self
    }

    /// Sets the active duration while preserving the rest of the timing configuration.
    #[must_use]
    pub fn duration(mut self, duration: impl Into<Duration>) -> Self {
        let timing = *self.keyframes.timing();

        self.keyframes = self
            .keyframes
            .with_timing(with_duration(timing, duration.into()));
        self
    }

    /// Sets the easing curve on the track timing.
    #[must_use]
    pub fn easing(mut self, easing: Easing) -> Self {
        let timing = self.keyframes.timing().with_easing(easing);

        self.keyframes = self.keyframes.with_timing(timing);
        self
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

    /// Samples the final keyframe state for this track.
    #[must_use]
    pub fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        self.keyframes.sample_at(1.0)
    }
}

impl PropertyTrackBuilder {
    /// Inserts the final value and returns the completed track.
    #[must_use]
    pub fn to(self, value: impl Into<PropertyValue>) -> Track {
        Track::new(self.keyframes.at(1.0, [(self.property, value.into())]))
    }
}

impl From<Keyframes> for Track {
    fn from(value: Keyframes) -> Self {
        Self::new(value)
    }
}

fn with_duration(timing: Timing, duration: Duration) -> Timing {
    Timing::new(duration.as_millis())
        .with_delay(timing.delay())
        .with_direction(timing.direction())
        .with_fill_mode(timing.fill_mode())
        .with_easing(timing.easing())
        .with_iterations(timing.iterations())
        .with_playback_rate(timing.playback_rate())
}
