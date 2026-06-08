//! Sequential, parallel, and fixed-duration animation composition.

mod hold;
mod parallel;
mod sequence;

pub use hold::Hold;
pub use parallel::Parallel;
pub use sequence::Sequence;

/// A sequential animation timeline.
pub type Timeline<T> = Sequence<T>;

fn normalized(progress: f32) -> f32 {
    if progress.is_nan() {
        0.0
    } else {
        progress.clamp(0.0, 1.0)
    }
}
