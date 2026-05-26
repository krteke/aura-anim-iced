//! Property-snapshot keyframe storage and lookup.

mod frame;
mod sample;
mod segment;
#[cfg(test)]
mod tests;
mod track;

pub use frame::{Keyframe, normalize_offset};
pub use segment::KeyframeSegment;
pub use track::Keyframes;
