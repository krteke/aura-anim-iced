//! Iced-first animation primitives for Aura UI experiments.

pub mod animatable;
pub mod iced_ext;
pub mod keyframes;
pub mod prelude;
pub mod property;
pub mod runtime;
pub mod timeline;
pub mod timing;

pub use animatable::Animatable;
pub use keyframes::Keyframes;
pub use property::{PropertyValue, UiProperty};
pub use runtime::AnimationRuntime;
pub use timeline::Timeline;
pub use timing::Timing;
