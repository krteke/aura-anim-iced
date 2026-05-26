//! Timeline orchestration primitives.

use crate::{keyframes::Keyframes, timing::Duration};

/// A named timeline marker at a fixed timeline offset.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineMarker {
    name: String,
    offset: Duration,
}

impl TimelineMarker {
    /// Creates a named marker at `offset`.
    #[must_use]
    pub fn new(name: impl Into<String>, offset: Duration) -> Self {
        Self {
            name: name.into(),
            offset,
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

/// A timeline step that can contribute duration to a timeline.
#[derive(Debug, Clone, PartialEq)]
pub enum TimelineStep {
    /// A keyframe track.
    Track(Track),
    /// A sequential group of steps.
    Sequence(Sequence),
    /// A parallel group of steps.
    Parallel(Parallel),
    /// A silent hold segment.
    Hold(Hold),
}

impl TimelineStep {
    /// Returns the finite total duration of this step, or `None` for infinite tracks.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        match self {
            Self::Track(track) => track.total_duration(),
            Self::Sequence(sequence) => sequence.total_duration(),
            Self::Parallel(parallel) => parallel.total_duration(),
            Self::Hold(hold) => Some(hold.total_duration()),
        }
    }
}

impl From<Track> for TimelineStep {
    fn from(value: Track) -> Self {
        Self::Track(value)
    }
}

impl From<Sequence> for TimelineStep {
    fn from(value: Sequence) -> Self {
        Self::Sequence(value)
    }
}

impl From<Parallel> for TimelineStep {
    fn from(value: Parallel) -> Self {
        Self::Parallel(value)
    }
}

impl From<Hold> for TimelineStep {
    fn from(value: Hold) -> Self {
        Self::Hold(value)
    }
}

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
}

/// A sequential timeline group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sequence {
    steps: Vec<TimelineStep>,
}

impl Sequence {
    /// Creates an empty sequence.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a sequence from steps.
    #[must_use]
    pub fn from_steps(steps: impl IntoIterator<Item = TimelineStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }

    /// Returns the steps in insertion order.
    #[must_use]
    pub fn steps(&self) -> &[TimelineStep] {
        &self.steps
    }

    /// Appends a timeline step.
    pub fn push_step(&mut self, step: impl Into<TimelineStep>) {
        self.steps.push(step.into());
    }

    /// Returns the finite sum of all step durations, or `None` if any step is infinite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        sum_durations(self.steps.iter().map(TimelineStep::total_duration))
    }
}

/// A parallel timeline group.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Parallel {
    steps: Vec<TimelineStep>,
}

impl Parallel {
    /// Creates an empty parallel group.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a parallel group from steps.
    #[must_use]
    pub fn from_steps(steps: impl IntoIterator<Item = TimelineStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }

    /// Returns the parallel steps in insertion order.
    #[must_use]
    pub fn steps(&self) -> &[TimelineStep] {
        &self.steps
    }

    /// Appends a timeline step.
    pub fn push_step(&mut self, step: impl Into<TimelineStep>) {
        self.steps.push(step.into());
    }

    /// Returns the finite maximum step duration, or `None` if any step is infinite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        max_duration(self.steps.iter().map(TimelineStep::total_duration))
    }
}

/// A silent timeline segment with a fixed duration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hold {
    duration: Duration,
}

impl Hold {
    /// Creates a hold segment.
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self { duration }
    }

    /// Returns the hold duration.
    #[must_use]
    pub const fn total_duration(self) -> Duration {
        self.duration
    }
}

/// A root timeline made of sequential steps.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Timeline {
    name: Option<String>,
    root: Sequence,
    markers: Vec<TimelineMarker>,
}

impl Timeline {
    /// Creates an empty timeline.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the timeline name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Returns the timeline name.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Returns the root sequence.
    #[must_use]
    pub const fn root(&self) -> &Sequence {
        &self.root
    }

    /// Returns the named markers in insertion order.
    #[must_use]
    pub fn markers(&self) -> &[TimelineMarker] {
        &self.markers
    }

    /// Appends a timeline step to the root sequence.
    pub fn push_step(&mut self, step: impl Into<TimelineStep>) {
        self.root.push_step(step);
    }

    /// Appends a named marker.
    pub fn push_marker(&mut self, marker: TimelineMarker) {
        self.markers.push(marker);
    }

    /// Returns the finite total duration of the root sequence, or `None` if any step is infinite.
    #[must_use]
    pub fn total_duration(&self) -> Option<Duration> {
        self.root.total_duration()
    }
}

fn sum_durations(durations: impl IntoIterator<Item = Option<Duration>>) -> Option<Duration> {
    let mut total_ms = 0.0;

    for duration in durations {
        total_ms += duration?.as_millis();
    }

    Some(Duration::from_millis(total_ms))
}

fn max_duration(durations: impl IntoIterator<Item = Option<Duration>>) -> Option<Duration> {
    let mut max_ms = 0.0_f64;

    for duration in durations {
        max_ms = max_ms.max(duration?.as_millis());
    }

    Some(Duration::from_millis(max_ms))
}

#[cfg(test)]
mod tests {
    use float_cmp::assert_approx_eq;

    use super::{Hold, Parallel, Sequence, Timeline, TimelineMarker, Track};
    use crate::{
        keyframes::Keyframes,
        timing::{Delay, Duration, IterationCount, Timing},
    };

    fn opacity_track(duration_ms: f64) -> Track {
        Track::new(
            Keyframes::new()
                .with_timing(Timing::new(duration_ms))
                .opacity(0.0, 0.0)
                .opacity(1.0, 1.0),
        )
    }

    #[test]
    fn timeline_starts_empty() {
        let timeline = Timeline::new();

        assert_eq!(timeline.name(), None);
        assert!(timeline.root().steps().is_empty());
        assert!(timeline.markers().is_empty());
        assert_approx_eq!(
            f64,
            timeline
                .total_duration()
                .expect("empty timeline duration")
                .as_millis(),
            0.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn tracks_use_keyframe_timing_total_duration() {
        let track = Track::new(
            Keyframes::new().with_timing(
                Timing::new(120.0)
                    .with_delay(Delay::from_millis(30.0))
                    .with_iterations(2),
            ),
        )
        .with_name("fade");

        assert_eq!(track.name(), Some("fade"));
        assert_approx_eq!(
            f64,
            track.total_duration().expect("track duration").as_millis(),
            270.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn sequence_duration_sums_step_durations() {
        let sequence = Sequence::from_steps([
            opacity_track(100.0).into(),
            Hold::new(Duration::from_millis(40.0)).into(),
            opacity_track(60.0).into(),
        ]);

        assert_eq!(sequence.steps().len(), 3);
        assert_approx_eq!(
            f64,
            sequence
                .total_duration()
                .expect("sequence duration")
                .as_millis(),
            200.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn parallel_duration_uses_longest_step() {
        let parallel = Parallel::from_steps([
            opacity_track(100.0).into(),
            Hold::new(Duration::from_millis(250.0)).into(),
            opacity_track(60.0).into(),
        ]);

        assert_eq!(parallel.steps().len(), 3);
        assert_approx_eq!(
            f64,
            parallel
                .total_duration()
                .expect("parallel duration")
                .as_millis(),
            250.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn timeline_duration_uses_root_sequence_and_markers_are_stored() {
        let mut timeline = Timeline::new().with_name("toast");
        timeline.push_step(Parallel::from_steps([
            opacity_track(80.0).into(),
            opacity_track(120.0).into(),
        ]));
        timeline.push_step(Hold::new(Duration::from_millis(300.0)));
        timeline.push_marker(TimelineMarker::new("settled", Duration::from_millis(120.0)));

        assert_eq!(timeline.name(), Some("toast"));
        assert_eq!(timeline.markers()[0].name(), "settled");
        assert_approx_eq!(
            f64,
            timeline.markers()[0].offset().as_millis(),
            120.0,
            epsilon = 1e-10
        );
        assert_approx_eq!(
            f64,
            timeline
                .total_duration()
                .expect("timeline duration")
                .as_millis(),
            420.0,
            epsilon = 1e-10
        );
    }

    #[test]
    fn infinite_track_makes_group_duration_infinite() {
        let infinite = Track::new(
            Keyframes::new()
                .with_timing(Timing::new(100.0).with_iterations(IterationCount::infinite())),
        );

        let sequence = Sequence::from_steps([opacity_track(40.0).into(), infinite.clone().into()]);
        let parallel = Parallel::from_steps([opacity_track(40.0).into(), infinite.into()]);

        assert_eq!(sequence.total_duration(), None);
        assert_eq!(parallel.total_duration(), None);
    }
}
