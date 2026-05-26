use super::Timeline;
use crate::{property::PropertySnapshot, timeline::error::TimelinePlaybackError, timing::Duration};

/// Timeline playback state controlled outside the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimelinePlaybackState {
    /// The timeline can advance and sample at the current position.
    #[default]
    Playing,
    /// The timeline is paused at the current position.
    Paused,
    /// The timeline was canceled and emits no sample.
    Canceled,
    /// The timeline is finished and emits the completion snapshot.
    Finished,
}

/// A sampled playback state.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePlaybackSnapshot {
    state: TimelinePlaybackState,
    position: Duration,
    properties: Option<PropertySnapshot>,
}

impl TimelinePlaybackSnapshot {
    /// Creates a playback snapshot.
    #[must_use]
    pub fn new(
        state: TimelinePlaybackState,
        position: Duration,
        properties: Option<PropertySnapshot>,
    ) -> Self {
        Self {
            state,
            position,
            properties,
        }
    }

    /// Returns the playback state.
    #[must_use]
    pub const fn state(&self) -> TimelinePlaybackState {
        self.state
    }

    /// Returns the sampled position.
    #[must_use]
    pub const fn position(&self) -> Duration {
        self.position
    }

    /// Returns the sampled property snapshot.
    #[must_use]
    pub const fn properties(&self) -> Option<&PropertySnapshot> {
        self.properties.as_ref()
    }
}

/// Playback controls for a timeline without owning runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimelinePlayback {
    state: TimelinePlaybackState,
    position: Duration,
}

impl TimelinePlayback {
    /// Creates playback controls at the start of a timeline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the current playback state.
    #[must_use]
    pub const fn state(&self) -> TimelinePlaybackState {
        self.state
    }

    /// Returns the current playback position.
    #[must_use]
    pub const fn position(&self) -> Duration {
        self.position
    }

    /// Moves playback to `position`.
    pub fn seek(&mut self, position: impl Into<Duration>) {
        self.position = position.into();
        if matches!(
            self.state,
            TimelinePlaybackState::Finished | TimelinePlaybackState::Canceled
        ) {
            self.state = TimelinePlaybackState::Playing;
        }
    }

    /// Pauses playback at the current position.
    pub fn pause(&mut self) {
        if self.state == TimelinePlaybackState::Playing {
            self.state = TimelinePlaybackState::Paused;
        }
    }

    /// Resumes playback at the current position.
    pub fn resume(&mut self) {
        if self.state == TimelinePlaybackState::Paused {
            self.state = TimelinePlaybackState::Playing;
        }
    }

    /// Cancels playback and suppresses future samples until seeking starts playback again.
    pub fn cancel(&mut self) {
        self.state = TimelinePlaybackState::Canceled;
    }

    /// Finishes playback and returns the completion snapshot for `timeline`.
    pub fn finish(
        &mut self,
        timeline: &Timeline,
    ) -> Result<TimelinePlaybackSnapshot, TimelinePlaybackError> {
        let Some(duration) = timeline.total_duration() else {
            return Err(TimelinePlaybackError::InfiniteTimeline);
        };

        self.position = duration;
        self.state = TimelinePlaybackState::Finished;

        Ok(self.snapshot(timeline))
    }

    /// Returns the current playback snapshot for `timeline`.
    #[must_use]
    pub fn snapshot(&self, timeline: &Timeline) -> TimelinePlaybackSnapshot {
        let properties = match self.state {
            TimelinePlaybackState::Playing | TimelinePlaybackState::Paused => {
                timeline.sample_at(self.position)
            }
            TimelinePlaybackState::Canceled => None,
            TimelinePlaybackState::Finished => timeline.completion_snapshot(),
        };

        TimelinePlaybackSnapshot::new(self.state, self.position, properties)
    }
}
