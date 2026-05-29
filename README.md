# aura-anim-iced

Iced-first animation orchestration for applications that need more than a single animated value.

This crate builds on Iced's public animation surface instead of replacing it.
User-facing APIs use Iced types such as `iced::Color`, `iced::Vector`,
`iced::Size`, `iced::Rectangle`, `iced::Shadow`, and
`iced::animation::Easing`. Internal interpolation helpers exist only to sample
multi-property keyframes, timelines, runtime snapshots, and diagnostics.

The v0.1 foundation focuses on:

- typed visual properties and sampled property snapshots;
- timing primitives that use Iced easing directly;
- property keyframes and timeline orchestration;
- a runtime that can gate Iced subscriptions while animations are active;
- Iced integration helpers for applying snapshots in `view` code.

Use Iced's `Animation<T>` for direct single-value animation. Use
`aura-anim-iced` when a UI state change needs coordinated opacity, transform,
size, color, shadow, hold, sequence, parallel, and runtime cleanup behavior.

## Status

`0.1.0-alpha.1` is an early foundation release. It focuses on typed property
snapshots, keyframes, timelines, runtime ticking, and Iced integration helpers.

## Installation

Add the crate to an Iced application:

```sh
cargo add aura-anim-iced
```

Enable optional diagnostics when runtime tick events should be visible through
`tracing`:

```sh
cargo add aura-anim-iced --features tracing
cargo add aura-anim-iced --features inspector
```

The same configuration can be written directly in `Cargo.toml`:

```toml
[dependencies]
aura-anim-iced = "0.1.0-alpha.1"
```

```toml
[dependencies]
aura-anim-iced = { version = "0.1.0-alpha.1", features = ["inspector"] }
```

## Minimal Runtime Example

Store an `AnimationRuntime` in application state, register keyframes in `update`,
keep an Iced tick subscription active while the runtime is playing,
and convert tick output into view effects for one target.

```rust
use std::time::Instant;

use aura_anim_iced::{iced_ext, prelude::*};

struct App {
    animations: AnimationRuntime,
    panel: AnimationTargetId,
    panel_effects: EffectSnapshot,
}

#[derive(Debug, Clone)]
enum Message {
    OpenPanel,
    AnimationTick(Instant),
}

fn update(app: &mut App, message: Message) {
    match message {
        Message::OpenPanel => {
            app.animations.register_keyframes(
                app.panel,
                Keyframes::new()
                    .with_timing(Timing::new(180.0))
                    .opacity(0.0, 0.0)
                    .opacity(1.0, 1.0)
                    .scale(0.0, 0.96)
                    .scale(1.0, 1.0),
            );
        }
        Message::AnimationTick(tick) => {
            let output = iced_ext::update_tick(&mut app.animations, tick);
            app.panel_effects = tick_effect_snapshot_for(&output, app.panel);
        }
    }
}

fn subscription(app: &App) -> iced::Subscription<Message> {
    iced_ext::subscription(&app.animations, Message::AnimationTick)
}
```

In `view`, apply the sampled `EffectSnapshot` fields to the widget style,
layout, or wrapper code owned by the application.
