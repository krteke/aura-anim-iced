//! Deferred animation factories that start from the current sampled value.

use crate::{
    spring::{Spring, SpringConfig},
    timing::Timing,
    traits::Animatable,
    tween::Tween,
};

/// Creates a deferred tween toward `target`.
///
/// The tween's starting value is sampled when it is played, making this
/// suitable for interrupted motions and independent field animations.
pub fn tween_to<T: Animatable>(target: T, timing: Timing) -> impl FnOnce(T) -> Tween<T> {
    move |from| Tween::between(from, target, timing)
}

/// Creates a deferred spring toward `target`.
///
/// The spring's starting value is sampled when it is played, making this
/// suitable for interrupted motions and independent field animations.
pub fn spring_to<T: Animatable>(target: T, config: SpringConfig) -> impl FnOnce(T) -> Spring<T> {
    move |from| Spring::new(from, target, config)
}
