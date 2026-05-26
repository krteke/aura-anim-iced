use crate::nearly_equal_f32;

use super::{Keyframe, normalize_offset};

/// A lookup result for a normalized keyframe offset.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyframeSegment<'a> {
    /// The track has no keyframes.
    Empty,
    /// The track has exactly one keyframe.
    Single(&'a Keyframe),
    /// The lookup offset resolves directly to one keyframe.
    Exact(&'a Keyframe),
    /// The lookup offset falls between two neighboring keyframes.
    Between {
        /// Keyframe before the lookup offset.
        from: &'a Keyframe,
        /// Keyframe after the lookup offset.
        to: &'a Keyframe,
        /// Normalized progress between `from` and `to`.
        progress: f32,
    },
}

impl<'a> KeyframeSegment<'a> {
    pub(crate) fn find(frames: &'a [Keyframe], offset: f32) -> Self {
        let offset = normalize_offset(offset);

        match frames {
            [] => Self::Empty,
            [single] => Self::Single(single),
            [first, ..] if offset <= first.offset() => Self::Exact(first),
            [.., last] if offset >= last.offset() => Self::Exact(last),
            _ => find_inside_track(frames, offset),
        }
    }

    /// Returns whether this segment can produce a sample without interpolation.
    #[must_use]
    pub const fn is_resolved(self) -> bool {
        matches!(self, Self::Single(_) | Self::Exact(_))
    }
}

fn find_inside_track(frames: &[Keyframe], offset: f32) -> KeyframeSegment<'_> {
    if let Some(exact) = frames
        .iter()
        .find(|frame| nearly_equal_f32(frame.offset(), offset))
    {
        return KeyframeSegment::Exact(exact);
    }

    for pair in frames.windows(2) {
        let from = &pair[0];
        let to = &pair[1];

        if from.offset() <= offset && offset <= to.offset() {
            return KeyframeSegment::Between {
                from,
                to,
                progress: segment_progress(from.offset(), to.offset(), offset),
            };
        }
    }

    KeyframeSegment::Empty
}

fn segment_progress(from: f32, to: f32, offset: f32) -> f32 {
    let span = to - from;

    if span > f32::EPSILON {
        ((offset - from) / span).clamp(0.0, 1.0)
    } else {
        0.0
    }
}
