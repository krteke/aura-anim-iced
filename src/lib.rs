//! Iced-first animation primitives.
//!
//! `aura-anim-iced` models advanced animation as sampled Iced UI properties
//! that can be applied from normal `update`, `subscription`, and `view` code.
//! The public API is Iced-first: user-facing values use Iced types and Iced's
//! animation primitives wherever they already exist.
//!
//! The foundation layer covers typed Iced UI properties, timing, property
//! keyframes, timelines, a small runtime, and Iced integration helpers. The
//! v0.2 behavior layer adds property-change transitions, state-driven
//! transitions, retargeting, interruption, and route screen transitions on top
//! of that base.
//!
//! Runtime integration follows a simple loop:
//!
//! 1. Store an [`AnimationRuntime`] in application state.
//! 2. Register keyframes or timelines when `update` receives user events.
//! 3. Subscribe to ticks only while the runtime is active.
//! 4. Sample snapshots on each tick and apply them while building `view`.
//!
//! Runnable examples live under `examples/`:
//!
//! - `animated_button.rs` for hover, press, focus, scale, color, and shadow
//!   animation.
//! - `keyframes_popup.rs` for opacity and scale keyframes.
//! - `timeline_toast.rs` for enter, hold, exit, and cleanup sequencing.
//! - `behavior_width.rs` for property-change animation from the current visual
//!   value.
//! - `route_transition.rs` for outgoing and incoming screen transition flow.

pub(crate) mod animatable;
pub mod behavior;
pub mod defaults;
pub mod iced_ext;
pub mod keyframes;
pub mod prelude;
pub mod property;
pub mod route;
pub mod runtime;
pub mod state;
pub mod timeline;
pub mod timing;

use crate::animatable::Animatable;
use crate::property::{PropertyValue, TransformValue};

const EPSILON_F32: f32 = 1e-5;
const EPSILON_F64: f64 = 1e-10;

pub(crate) fn nearly_equal_f64(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON_F64
}

pub(crate) fn nearly_equal_f32(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON_F32
}

fn interpolate_value(
    from: PropertyValue,
    to: PropertyValue,
    progress: f32,
) -> Option<PropertyValue> {
    match (from, to) {
        (PropertyValue::Scalar(from), PropertyValue::Scalar(to)) => {
            Some(PropertyValue::Scalar(f32::interpolate(from, to, progress)))
        }
        (PropertyValue::Vector2(from), PropertyValue::Vector2(to)) => Some(PropertyValue::Vector2(
            iced::Vector::interpolate(from, to, progress),
        )),
        (PropertyValue::Size(from), PropertyValue::Size(to)) => Some(PropertyValue::Size(
            iced::Size::interpolate(from, to, progress),
        )),
        (PropertyValue::Rectangle(from), PropertyValue::Rectangle(to)) => Some(
            PropertyValue::Rectangle(iced::Rectangle::interpolate(from, to, progress)),
        ),
        (PropertyValue::Transform(from), PropertyValue::Transform(to)) => Some(
            PropertyValue::Transform(TransformValue::interpolate(from, to, progress)),
        ),
        (PropertyValue::Color(from), PropertyValue::Color(to)) => Some(PropertyValue::Color(
            iced::Color::interpolate(from, to, progress),
        )),
        (PropertyValue::Shadow(from), PropertyValue::Shadow(to)) => Some(PropertyValue::Shadow(
            iced::Shadow::interpolate(from, to, progress),
        )),
        _ => None,
    }
}
