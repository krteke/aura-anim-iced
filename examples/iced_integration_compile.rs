//! Compile-only Iced integration example.

use std::time::Instant;

use aura_anim_iced::{iced_ext, prelude::*};

fn animation_tick(_: Instant) {}

fn main() {
    let mut runtime = AnimationRuntime::testing();

    let _subscription = iced_ext::subscription(&runtime, animation_tick);

    runtime.register_keyframes(
        Keyframes::new()
            .with_timing(Timing::new(100.0))
            .opacity(0.0, 0.0)
            .opacity(1.0, 1.0),
    );

    runtime.clock_mut().set_now(Duration::from_millis(50.0));
    let tick = iced_ext::update_tick(&mut runtime, Instant::now());
    let _effects = tick_effect_snapshot(&tick);
}
