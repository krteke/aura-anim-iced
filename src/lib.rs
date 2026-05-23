//! Iced-first animation primitives and integration helpers.
//!
//! This crate is intentionally scaffolded around explicit animation state,
//! deterministic sampling, and Iced integration. Public implementation APIs
//! will be added module by module as the v0.1 scope is built.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(unused_must_use)]

pub mod animated;
pub mod clock;
pub mod easing;
pub mod iced_ext;
pub mod interpolate;
pub mod timeline;
pub mod transition;
pub mod tween;
pub mod value;
#[cfg(feature = "widgets")]
#[cfg_attr(docsrs, doc(cfg(feature = "widgets")))]
pub mod widget;

pub use easing::Easing;
pub use interpolate::Interpolate;

#[cfg(feature = "testing")]
#[cfg_attr(docsrs, doc(cfg(feature = "testing")))]
pub mod testing;
