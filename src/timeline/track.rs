use crate::{
    keyframes::{Keyframes, KeyframesBuilder},
    prelude::PropertyValueKind,
    property::{PropertyEntry, PropertySnapshot, PropertySpec},
    timing::{Duration, Easing, Timing},
};

/// A keyframe track placed in a timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    name: Option<String>,
    keyframes: Keyframes,
}

/// A builder for creating a single-property timeline track.
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTrackBuilder<K: PropertyValueKind> {
    property: PropertySpec<K>,
    builder: KeyframesBuilder,
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
    pub fn from<K: PropertyValueKind>(
        property: PropertySpec<K>,
        value: K::Inner,
    ) -> PropertyTrackBuilder<K> {
        PropertyTrackBuilder {
            property,
            builder: KeyframesBuilder::new().at(0.0, (property, value)),
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
        let mut snapshot = PropertySnapshot::with_capacity(self.keyframes.track_count());

        if self.sample_into(offset, &mut snapshot) {
            Some(snapshot)
        } else {
            None
        }
    }

    pub(crate) fn sample_into(
        &self,
        offset: impl Into<Duration>,
        output: &mut PropertySnapshot,
    ) -> bool {
        let timing = self
            .keyframes
            .timing()
            .normalize_elapsed(offset.into().as_millis());

        if !timing.has_sample() {
            output.clear();
            return false;
        }

        #[allow(
            clippy::cast_possible_truncation,
            reason = "Normalized keyframe offsets are stored as f32 throughout the keyframe module."
        )]
        self.keyframes
            .sample_into(timing.iteration_progress as f32, output)
    }

    /// Samples the final keyframe state for this track.
    #[must_use]
    pub fn completion_snapshot(&self) -> Option<PropertySnapshot> {
        self.keyframes.sample_completion()
    }
}

impl<K: PropertyValueKind> PropertyTrackBuilder<K> {
    /// Creates an empty builder for `property`.
    #[must_use]
    pub fn new(property: PropertySpec<K>) -> Self {
        Self {
            property,
            builder: KeyframesBuilder::default(),
        }
    }

    /// Inserts the final value and returns the completed track.
    #[must_use]
    pub fn to(self, value: K::Inner) -> Self {
        let property = self.property;
        let builder = self.builder.at(1.0, (property, value));

        Self { property, builder }
    }

    /// Inserts a keyframe at the end of the track with the given property and value.
    #[must_use]
    pub fn keyframe_at_end(mut self, property: PropertyEntry) -> Self {
        self.builder = self.builder.at(1.0, vec![property]);
        self
    }

    /// Sets the active duration while preserving the rest of the timing configuration.
    #[must_use]
    pub fn duration(mut self, duration: impl Into<Duration>) -> Self {
        let timing = *self.builder.timing();

        self.builder = self
            .builder
            .with_timing(with_duration(timing, duration.into()));
        self
    }

    /// Sets the easing curve on the track timing.
    #[must_use]
    pub fn easing(mut self, easing: Easing) -> Self {
        let timing = self.builder.timing().with_easing(easing);

        self.builder = self.builder.with_timing(timing);
        self
    }

    /// Finishes the builder and returns a timeline track.
    #[must_use]
    pub fn finish(self) -> Track {
        Track::new(self.builder.finish())
    }
}

impl<K: PropertyValueKind> From<PropertyTrackBuilder<K>> for Track {
    fn from(value: PropertyTrackBuilder<K>) -> Self {
        value.finish()
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
