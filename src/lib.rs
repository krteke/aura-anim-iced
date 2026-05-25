//! Iced-first animation primitives.
//!
//! `aura-anim-iced` models animation as sampled UI properties that can be
//! applied from normal Iced `update`, `subscription`, and `view` code. The
//! crate keeps animation state outside widgets: keyframes and timelines produce
//! property snapshots, the runtime advances active animations, and Iced-specific
//! helpers turn those snapshots into view-layer effects.
//!
//! The v0.1 scope is the foundation layer: animatable values, typed UI
//! properties, timing, keyframes, timelines, a small runtime, and optional Iced
//! integration behind the `iced` feature. Higher-level behaviors, gestures,
//! layout transitions, theme transitions, and inspector tooling are planned as
//! feature-gated layers on top of this base.
//!
//! Runtime integration follows a simple loop:
//!
//! 1. Store an [`AnimationRuntime`] in application state.
//! 2. Register keyframes or timelines when `update` receives user events.
//! 3. Subscribe to ticks only while the runtime is active.
//! 4. Sample snapshots on each tick and apply them while building `view`.
//!
//! Planned v0.1 examples will live under `examples/`:
//!
//! - `animated_button.rs` for hover, press, focus, scale, color, and shadow
//!   animation.
//! - `keyframes_popup.rs` for opacity and scale keyframes.
//! - `timeline_toast.rs` for enter, hold, exit, and cleanup sequencing.

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
pub use property::{
    PropertyCompositionKey, PropertyValue, PropertyValueError, PropertyValueKind, RectangleValue,
    SizeValue, TransformValue, UiProperty, Vector2Value, sort_properties_by_composition,
    sort_property_entries_by_composition,
};
pub use runtime::AnimationRuntime;
pub use timeline::Timeline;
pub use timing::Timing;
