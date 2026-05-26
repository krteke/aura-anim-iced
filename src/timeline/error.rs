/// Errors that can occur during timeline playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelinePlaybackError {
    /// The timeline has infinite duration and cannot be played.
    InfiniteTimeline,
}

impl std::fmt::Display for TimelinePlaybackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InfiniteTimeline => write!(f, "timeline has infinite duration"),
        }
    }
}

impl std::error::Error for TimelinePlaybackError {}
