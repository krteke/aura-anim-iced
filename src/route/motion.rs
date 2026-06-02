use crate::{
    property::{OPACITY, TRANSLATE},
    timeline::{Parallel, Timeline, Track},
    timing::Duration,
};

/// Built-in incoming screen motion made of opacity and translate tracks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RouteIncomingMotion {
    offset: iced::Vector,
    duration: Duration,
}

impl RouteIncomingMotion {
    /// Creates incoming screen motion from `offset` to the resting position.
    #[must_use]
    pub const fn new(offset: iced::Vector, duration: Duration) -> Self {
        Self { offset, duration }
    }

    /// Returns the initial translation offset.
    #[must_use]
    pub const fn offset(&self) -> iced::Vector {
        self.offset
    }

    /// Returns the incoming motion duration.
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// Builds the incoming opacity and position timeline.
    #[must_use]
    pub fn timeline(&self) -> Timeline {
        Timeline::new().then(
            Parallel::new()
                .track(Track::from(OPACITY, 0.0).to(1.0).duration(self.duration))
                .track(
                    Track::from(TRANSLATE, self.offset)
                        .to(iced::Vector::new(0.0, 0.0))
                        .duration(self.duration),
                ),
        )
    }
}
