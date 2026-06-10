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
aura-anim = "0.2.2"
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
    Timing::ease_out(160.0),
);

button.transition_to(
    ButtonMotion {
        opacity: 1.0,
        scale: 1.0,
    },
    &mut runtime,
)
.unwrap();

runtime.tick(std::time::Duration::from_millis(80));
let visual = button.value(&runtime).unwrap();
```

`transition_to` retargets from the currently sampled value, so interrupted
hover, press, menu and route animations do not jump back to a stale origin.
Motion access and mutation return `Result<_, MotionError>` so removed, stale,
out-of-bounds, and type-mismatched handles remain distinguishable.

`Timing::linear`, `Timing::ease_in`, `Timing::ease_out`, and
`Timing::ease_in_out` cover the common duration/easing combinations while
remaining normal `Timing` values that can be extended with delay, iterations,
or direction.

Use a deferred target factory when replacing the current animation:

```rust
button.play(
    tween_to(
        ButtonMotion {
            opacity: 0.0,
            scale: 0.9,
        },
        Timing::ease_in(120.0),
    ),
    &mut runtime,
)?;
# Ok::<(), MotionError>(())
```

`tween_to` and `spring_to` sample the motion's current value when playback
starts, so callers do not need to read or clone it manually.

## Independent Field Animations

`#[derive(Animatable)]` also generates typed field descriptors. A struct can
remain one `Motion<T>` while each selected field uses a different animation:

```rust
#[derive(Clone, Debug, Animatable)]
struct Position {
    x: f32,
    y: f32,
}

let mut runtime = MotionRuntime::new();
let position = runtime.motion(Position { x: 0.0, y: 0.0 });

position.play(
    fields()
        .animate(
            field!(Position::x),
            tween_to(100.0, Timing::ease_in(100.0)),
        )
        .animate(
            PositionFields::y,
            spring_to(200.0, SpringConfig::snappy()),
        ),
    &mut runtime,
)?;
# Ok::<(), MotionError>(())
```

Target factories receive the field's current sampled value when `play` is
called. Custom `|from| ...` factories remain supported. Interrupted field
animations therefore continue from the visible value, while fields not
included in the plan retain their current values.

For a named struct, the derive generates `PositionFields::x`,
`PositionFields::y`, and equivalent `field!(Position::x)` descriptors. Tuple
struct descriptors use `_0`, `_1`, and so on, while `field!(Offset::0)` uses
the tuple index directly. A generated descriptor type can be renamed when
necessary:

```rust
#[derive(Clone, Animatable)]
#[animatable(fields = PositionAnimationFields)]
struct Position {
    x: f32,
    y: f32,
}
```

## Animation Events

`MotionRuntime` queues structured lifecycle events when playbacks complete,
cancel, are interrupted, or leave runtime storage. Events are emitted once per
state transition and remain queued until the application takes or clears them:

```rust
aura_anim::iced::frame(&mut runtime, now);

for event in runtime.take_events() {
    if event.is_completed_for(motion) {
        // Run one-time completion logic.
    }
}
```

Matching a `Motion<T>` is sufficient for simple cleanup. Multi-stage flows
should track the exact playback so an older queued event cannot complete a
newer animation on the same motion:

```rust
let exit = motion.play_tracked(
    Tween::between(current, hidden, Timing::new(150.0)),
    &mut runtime,
)?;

// In a later frame update:
for event in runtime.take_events() {
    if event.is_completed_for(exit) {
        let enter = motion.play_tracked(
            Tween::between(hidden, visible, Timing::new(220.0)),
            &mut runtime,
        )?;
        # let _ = enter;
    }
}
# Ok::<(), MotionError>(())
```

`play_tracked` and `transition_to_tracked` return a `PlaybackId`. Existing
`play` and `transition_to` calls remain unchanged when playback identity is not
needed.

Event kinds include:

- `Completed`
- `Canceled`
- `Interrupted(Replaced | Retargeted | Removed)`
- `Removed(Explicit | Settled)`

`DropWhenSettled` motions emit their terminal event before their removal event,
so completion remains observable after the handle becomes invalid.

`Presence::handle_event` uses the current exit playback ID to avoid stale exit
events unmounting content that has already been shown again:

```rust
for event in runtime.take_events() {
    menu.handle_event(&event);
    toast.handle_event(&event);
}
```

`Presence::sync` remains available for polling-based integrations.

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
                if let Err(error) = self.panel.transition_to(
                    PanelMotion {
                        opacity: 1.0,
                        offset: Vector::ZERO,
                    },
                    &mut self.runtime,
                ) {
                    eprintln!("panel transition failed: {error}");
                }
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

## Motion Binding

`MotionBinding<S, T>` maps reusable business states to visual targets and
transition factories. The binding is immutable configuration; each button,
menu item, or route owns a small `MotionBindingState<S>` that records its last
successfully applied state.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ButtonState {
    Idle,
    Hovered,
    Pressed,
}

let button_binding = MotionBinding::new(ButtonState::Idle, idle)
    .when(ButtonState::Hovered, hovered)
    .when(ButtonState::Pressed, pressed)
    .transition(ButtonState::Idle, ButtonState::Hovered, |ctx| {
        ctx.tween(Timing::new(120.0))
    })
    .transition(ButtonState::Hovered, ButtonState::Pressed, |ctx| {
        ctx.spring(SpringConfig::snappy())
    })
    .fallback(|ctx| ctx.tween(Timing::new(100.0)));

let motion = runtime.motion(idle);
let mut binding_state = button_binding.state();

button_binding.set_state(
    &mut binding_state,
    ButtonState::Hovered,
    motion,
    &mut runtime,
)?;
```

On each state change the binding resolves the target, samples the motion's
current value, selects the exact transition or fallback factory, constructs
the animation, calls `motion.play(...)`, and records the new business state
only after playback succeeds. Factories can return concrete Tween, Spring,
Keyframes, Timeline, or any custom `Animation<T>`; the binding handles type
erasure internally.

One binding configuration can be cloned or shared and reused with independent
`MotionBindingState` values.

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
aura-anim = "0.2.2"
```

For Oklab RGB interpolation with independently interpolated alpha:

```toml
aura-anim = {
    version = "0.2.2",
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

## Tracing

Enable the optional `tracing` feature to emit runtime diagnostics without
installing or configuring a subscriber inside the library:

```toml
aura-anim = {
    version = "0.2.2",
    features = ["tracing"]
}
```

The `aura_anim::runtime`, `aura_anim::binding`, and `aura_anim::presence`
targets report motion allocation and reuse, playback commands, lifecycle
changes, invalid handles, binding transition selection, and presence mounting.
Per-tick diagnostics use the `TRACE` level; lifecycle and error diagnostics use
`DEBUG`. Applications remain responsible for installing a compatible
`tracing` subscriber.

## Animation Sources

### Tween

```rust
motion.play(
    Tween::between(current, target, Timing::new(180.0)).rate(2.0),
    &mut runtime,
);
```

Timing supports delay, easing, finite or infinite iterations, and playback
direction. `Animation::rate` directly scales stored durations: `2.0` halves
duration and `0.5` doubles it. It recursively updates existing Timeline
children, while Spring ignores rate because its motion is physics-based.

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

For values whose fields need different physical responses, create independent
spring channels and explicitly compose their outputs:

```rust
#[derive(Clone, Debug, Animatable)]
struct PanelMotion {
    offset: f32,
    opacity: f32,
}

let movement = SpringConfig {
    stiffness: 180.0,
    damping: 20.0,
    ..SpringConfig::default()
};
let fade = SpringConfig {
    stiffness: 420.0,
    damping: 32.0,
    ..SpringConfig::default()
};

let spring = Spring::with_channels(
    PanelMotion {
        offset: 24.0,
        opacity: 0.0,
    },
    PanelMotion {
        offset: 0.0,
        opacity: 1.0,
    },
    [movement, fade],
    |outputs| PanelMotion {
        offset: outputs[0].offset,
        opacity: outputs[1].opacity,
    },
);
```

Each channel owns its own position, velocity and `SpringConfig`. Spring
advancement uses the analytic damped-oscillator solution, so long frame
intervals are fully consumed instead of being truncated.

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
