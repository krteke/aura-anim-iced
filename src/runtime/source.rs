use crate::{keyframes::Keyframes, timeline::Timeline};

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
