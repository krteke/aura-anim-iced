use crate::timing::Duration;

/// A named timeline marker at a fixed timeline offset.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMarker {
    name: String,
    offset: Duration,
}

impl TimelineMarker {
    /// Creates a named marker at `offset`.
    #[must_use]
    pub fn new(name: impl Into<String>, offset: impl Into<Duration>) -> Self {
        Self {
            name: name.into(),
            offset: offset.into(),
        }
    }

    /// Returns the marker name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the marker offset.
    #[must_use]
    pub const fn offset(&self) -> Duration {
        self.offset
    }
}
