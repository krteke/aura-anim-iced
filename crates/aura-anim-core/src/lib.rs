//! Core animation primitives, timing types, composition utilities, and runtime storage.

pub mod binding;
pub mod field;
#[cfg(feature = "iced")]
mod iced;
pub mod interpolate;
pub mod keyframes;
pub mod presence;
pub mod runtime;
pub mod spring;
pub mod target;
pub mod timeline;
pub mod timing;
pub mod traits;
pub mod tween;

/// Re-exports the `Animatable` and `field` macros from `aura_anim_macros`.
pub mod macros {
    pub use aura_anim_macros::{Animatable, field};
}
