//! Iced-first animation primitives.
//!
//! `aura-anim-iced` models advanced animation as sampled Iced UI properties
//! that can be applied from normal `update`, `subscription`, and `view` code.
//! The public API is Iced-first: user-facing values use Iced types and Iced's
//! animation primitives wherever they already exist. The crate keeps its own
//! pure calculation modules only where they help orchestrate multi-property
//! keyframes, timelines, snapshots, and diagnostics.
//!
//! The v0.1 scope is the foundation layer: typed Iced UI properties, timing,
//! property keyframes, timelines, a small runtime, and Iced integration helpers.
//! Higher-level behaviors, gestures, layout transitions, theme transitions, and
//! inspector tooling are planned as layers on top of this base.
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

#[allow(dead_code)]
pub(crate) mod animatable;
pub mod iced_ext;
pub mod keyframes;
pub mod prelude;
pub mod property;
pub mod runtime;
pub mod timeline;
pub mod timing;

pub use keyframes::{Keyframe, Keyframes};
pub use property::{
    PropertyCompositionKey, PropertySnapshot, PropertyValue, PropertyValueError, PropertyValueKind,
    TransformValue, UiProperty, sort_properties_by_composition,
    sort_property_entries_by_composition,
};
pub use runtime::AnimationRuntime;
pub use timeline::Timeline;
pub use timing::{
    Delay, Direction, Duration, Easing, FillMode, IterationCount, NormalizedTiming, Timing,
    TimingPhase, TimingSampleState,
};
