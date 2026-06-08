# aura-anim

Typed animation primitives and Iced integration for Rust desktop interfaces.

The application stores lightweight `Motion<T>` handles while `MotionRuntime`
owns and advances the actual animation sources. Animated values remain ordinary
Rust structs, and `#[derive(Animatable)]` generates field-by-field
interpolation.

```text
Application
├── explicit UI state
├── Motion<T> handles
└── event-driven transition_to / play calls

MotionRuntime
├── owns type-erased animation slots
├── ticks active slots only
├── pause / resume / seek / cancel / finish
├── generation-checked handle reuse
└── completion compaction and optional auto-removal

Animation<T>
├── Tween<T>
├── Spring<T>
├── Keyframes<T>
├── Sequence<T>
├── Parallel<T>
└── Hold<T>
```

## Workspace Crates

- `aura-anim-core`: runtime, handles, interpolation, animation sources and
  timeline composition.
- `aura-anim-iced`: Iced value integration and frame subscriptions.
- `aura-anim`: convenience facade re-exporting core and Iced APIs.
- `aura-anim-macros`: `Animatable` derive implementation.

## Installation

For Iced applications:

```toml
[dependencies]
aura-anim = "0.2.1"
iced = "0.14"
```

Use `aura-anim-core` directly when no Iced integration is required.

## Typed Motion

```rust
use aura_anim::prelude::*;

#[derive(Clone, Debug, Animatable)]
struct ButtonMotion {
    opacity: f32,
    scale: f32,
}

let mut runtime = MotionRuntime::new();
let button = runtime.motion_with(
    ButtonMotion {
        opacity: 0.5,
        scale: 0.95,
    },
    Timing::new(160.0).with_easing(Easing::EaseOut),
);

button.transition_to(
    ButtonMotion {
        opacity: 1.0,
        scale: 1.0,
    },
    &mut runtime,
);

runtime.tick(std::time::Duration::from_millis(80));
let visual = button.value(&runtime);
```

`transition_to` retargets from the currently sampled value, so interrupted
hover, press, menu and route animations do not jump back to a stale origin.

## Iced Integration

Store the runtime and typed handles in application state:

```rust
use std::time::Instant;

use aura_anim::prelude::*;
use iced::{Subscription, Vector};

#[derive(Clone, Debug, Animatable)]
struct PanelMotion {
    opacity: f32,
    offset: Vector,
}

struct App {
    runtime: MotionRuntime,
    panel: Motion<PanelMotion>,
}

#[derive(Clone, Debug)]
enum Message {
    Frame(Instant),
    Open,
}

impl App {
    fn update(&mut self, message: Message) {
        match message {
            Message::Frame(now) => aura_anim::iced::frame(&mut self.runtime, now),
            Message::Open => {
                self.panel.transition_to(
                    PanelMotion {
                        opacity: 1.0,
                        offset: Vector::ZERO,
                    },
                    &mut self.runtime,
                );
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        aura_anim::iced::subscription_with_policy(
            &self.runtime,
            TickPolicy::fps(60),
        )
        .map(Message::Frame)
    }
}
```

When no animation is active, the subscription returns `Subscription::none()`
and does not continue waking the application.

`TickPolicy` supports:

```rust
TickPolicy::Frames
TickPolicy::fps(60)
TickPolicy::interval(std::time::Duration::from_millis(32))
```

## Iced Animatable Types

With the core `iced` integration enabled, these types can be fields in an
`Animatable` struct:

- `iced::Vector<T>`
- `iced::Point<T>`
- `iced::Size<T>`
- `iced::Rectangle<T>`
- `iced::Padding`
- `iced::border::Radius`

The active `rgba` or `oklaba` color feature additionally enables:

- `iced::Color`
- `iced::Shadow`
- `iced::Border`

## Color Interpolation

RGBA component interpolation is enabled by default:

```toml
aura-anim = "0.2.1"
```

For Oklab RGB interpolation with independently interpolated alpha:

```toml
aura-anim = {
    version = "0.2.1",
    default-features = false,
    features = ["oklaba"]
}
```

`rgba` and `oklaba` are mutually exclusive. Oklaba conversion follows:

```text
Iced sRGB
→ palette sRGB
→ Oklab interpolation
→ display sRGB
```

## Animation Sources

### Tween

```rust
motion.play(
    Tween::between(current, target, Timing::new(180.0)),
    &mut runtime,
);
```

Timing supports delay, easing, finite or infinite iterations, and playback
direction.

### Keyframes

```rust
motion.play(
    Keyframes::new(start)
        .push_eased(180.0, overshoot, Easing::EaseOut)
        .push_eased(280.0, settled, Easing::EaseInOut),
    &mut runtime,
);
```

### Spring

```rust
motion.play(
    Spring::new(current, target, SpringConfig::default()),
    &mut runtime,
);
```

Spring interpolation may overshoot and can be retargeted while active.

## Timeline Composition

`Sequence`, `Parallel` and `Hold` all implement `Animation<T>`, so composition
is recursive:

```text
Sequence(
    Parallel(
        Sequence(Hold, Tween),
        Sequence(Tween, Tween),
    ),
    Tween,
)
```

Parallel branches produce complete `T` values. A compositor explicitly selects
which fields each branch owns:

```rust
let parallel = Parallel::new(start.clone(), |outputs: &[Position]| Position {
    x: outputs[0].x,
    y: outputs[1].y,
})
.with(x_sequence)
.with(y_sequence);
```

Sequence propagates unused frame time into following children. Parallel
completes when its longest branch completes.

## Lifecycle

Normal motions retain their final value:

```rust
let motion = runtime.motion(initial);
```

Completed sources are compacted to the final value, releasing keyframe and
timeline trees while keeping the handle valid.

Transient animations can remove their slot automatically:

```rust
let transient = runtime.play_once(animation);
```

Slots are reused with generation counters, preventing stale handles from
accessing a newly allocated motion.

## Examples

Run the command-line architecture example:

```sh
cargo run -p aura-anim --example runtime
```

Run the Iced showcase:

```sh
cargo run -p aura-anim-iced --example showcase
```

Run the focused visual examples:

```sh
cargo run -p aura-anim-iced --example tween
cargo run -p aura-anim-iced --example keyframes
cargo run -p aura-anim-iced --example timeline
cargo run -p aura-anim-iced --example spring
```

Run the interactive UI examples:

```sh
cargo run -p aura-anim-iced --example button
cargo run -p aura-anim-iced --example menu
cargo run -p aura-anim-iced --example notification
cargo run -p aura-anim-iced --example route_transition
```

Run the showcase with perceptual color interpolation:

```sh
cargo run -p aura-anim-iced \
    --no-default-features \
    --features oklaba \
    --example showcase
```
