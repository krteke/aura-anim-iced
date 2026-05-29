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

## Animatable Values

Public animation inputs use Iced value types wherever possible. The v0.1 value
model covers scalar values, `iced::Vector`, `iced::Size`, `iced::Rectangle`,
`iced::Color`, `iced::Shadow`, and transform-friendly values. Interpolation is
kept internal so application code works with typed properties and sampled
snapshots instead of implementing animation traits.

```rust
use aura_anim_iced::{Keyframes, Timing, property};
use iced::Color;

let fade_and_color = Keyframes::new()
    .with_timing(Timing::new(160.0))
    .at(0.0, (property::OPACITY, 0.0))
    .at(1.0, (property::OPACITY, 1.0))
    .at(0.0, (property::BACKGROUND, Color::from_rgb(0.12, 0.14, 0.18)))
    .at(1.0, (property::BACKGROUND, Color::from_rgb(0.20, 0.36, 0.52)));
```

## Property Tracks

Properties are identified by typed `PropertySpec` values. Built-in specs cover
opacity, translation, scale, size, padding, radius, colors, and shadow.
Applications can also define custom specs when an example or widget needs an
extra sampled value, such as a toast offset.

```rust
use aura_anim_iced::{PropertyKey, PropertySpec, property};

const TOAST_Y: PropertySpec<property::Scalar> =
    PropertySpec::new(PropertyKey::new("app", "toast-y"), 21);
```

A `PropertySnapshot` stores sampled values for one target. When snapshots are
merged, later values replace earlier values with the same property spec and the
result is sorted by composition order.

## Keyframes

`Keyframes` store property snapshots at normalized offsets from `0.0` to `1.0`.
Each keyframe track owns a `Timing`, so duration, easing, fill mode, direction,
iterations, and playback rate stay attached to the sampled property data.

```rust
use aura_anim_iced::{Easing, Keyframes, Timing, property};

let popup_open = Keyframes::new()
    .with_timing(Timing::new(280.0).with_easing(Easing::EaseOut))
    .at(0.0, (property::OPACITY, 0.0))
    .at(0.0, (property::SCALE, 0.92))
    .at(0.68, (property::SCALE, 1.07))
    .at(1.0, (property::OPACITY, 1.0))
    .at(1.0, (property::SCALE, 1.0));
```

Duplicate offsets are merged. If the same property appears multiple times at
the same offset, the later value wins.

## Timeline Orchestration

Timelines combine keyframe tracks into sequences, parallel groups, and holds.
Use sequences for lifecycle animation, parallel groups for coordinated property
changes, and holds when a state should remain visible before the next step.

```rust
use aura_anim_iced::{Duration, Easing, Hold, Keyframes, Timeline, Timing, Track, property};

let enter = Track::new(
    Keyframes::new()
        .with_timing(Timing::new(220.0).with_easing(Easing::EaseOut))
        .at(0.0, (property::OPACITY, 0.0))
        .at(1.0, (property::OPACITY, 1.0)),
);

let exit = Track::new(
    Keyframes::new()
        .with_timing(Timing::new(180.0).with_easing(Easing::EaseIn))
        .at(0.0, (property::OPACITY, 1.0))
        .at(1.0, (property::OPACITY, 0.0)),
);

let toast_lifecycle = Timeline::sequence([
    enter.into(),
    Hold::new(Duration::from_millis(1_200.0)).into(),
    exit.into(),
]);
```

Use `Timeline::parallel` when several tracks should sample at the same time.
Property collisions are resolved by insertion order inside the target snapshot.

## Runtime Ticking

`AnimationRuntime` stores active keyframes and timelines by target ID. Register a
source in `update`, keep the returned handle if completion cleanup matters, and
route tick output back into application state.

```rust
use aura_anim_iced::{AnimationRuntime, AnimationTargetId, Keyframes, Timing, property};

let mut runtime = AnimationRuntime::new();
let target = AnimationTargetId::new();

let registration = runtime.register_keyframes(
    target,
    Keyframes::new()
        .with_timing(Timing::new(120.0))
        .at(0.0, (property::OPACITY, 0.0))
        .at(1.0, (property::OPACITY, 1.0)),
);

let handle = registration.handle();
```

Each runtime tick returns target-scoped snapshots plus completed handles.
Completed entries are removed automatically after their final output is emitted.

## Iced Subscription Wiring

Use `iced_ext::subscription` to produce ticks only while the runtime has active
animations. Use `iced_ext::update_tick` to advance the runtime from an Iced tick
message. The runtime tick interval comes from `TickPolicy`.

```rust
use std::time::Instant;

use aura_anim_iced::{AnimationRuntime, iced_ext};

#[derive(Debug, Clone)]
enum Message {
    AnimationTick(Instant),
}

fn subscription(runtime: &AnimationRuntime) -> iced::Subscription<Message> {
    iced_ext::subscription(runtime, Message::AnimationTick)
}
```

For view code, convert tick output with `tick_effect_snapshot_for` when using
the built-in effect fields, or read `AnimationTick::properties_for` directly
when the application owns custom property specs.
