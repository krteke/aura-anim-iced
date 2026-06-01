# aura-anim-iced

Iced-first animation orchestration for applications that need coordinated
property changes, state transitions, and screen-to-screen route motion.

This crate builds on Iced's public animation surface instead of replacing it.
User-facing APIs use Iced types such as `iced::Color`, `iced::Vector`,
`iced::Size`, `iced::Rectangle`, `iced::Shadow`, and
`iced::animation::Easing`.

The foundation layer covers:

- typed visual properties and sampled property snapshots;
- timing primitives that use Iced easing directly;
- property keyframes and timeline orchestration;
- a runtime that can gate Iced subscriptions while animations are active;
- Iced integration helpers for applying snapshots in `view` code.

The v0.2 behavior layer adds:

- `PropertyTransition` and `BehaviorRule` for animating value changes from the
  visual value currently on screen;
- `StateAnimator`, `StateTransition`, and `StateTransitionSet` for mapping
  application state changes to timelines;
- retargeting and interruption helpers that replace active animations without
  jumping back to stale target values;
- `RouteAnimator` and route screen transitions for outgoing and incoming route
  motion on separate targets.

Use Iced's `Animation<T>` for direct single-value animation. Use
`aura-anim-iced` when a UI state change needs coordinated opacity, transform,
size, color, shadow, hold, sequence, parallel, and runtime cleanup behavior.

## Status

v0.2.0-alpha

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
aura-anim-iced = "0.2.0"
```

```toml
[dependencies]
aura-anim-iced = { version = "0.2.0", features = ["inspector"] }
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
                KeyframesBuilder::new()
                    .with_timing(Timing::new(180.0))
                    .opacity(0.0, 0.0)
                    .opacity(1.0, 1.0)
                    .scale(0.0, 0.96)
                    .scale(1.0, 1.0)
                    .finish(),
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
use aura_anim_iced::{KeyframesBuilder, Timing, property};
use iced::Color;

let fade_and_color = KeyframesBuilder::new()
    .with_timing(Timing::new(160.0))
    .at(0.0, (property::OPACITY, 0.0))
    .at(1.0, (property::OPACITY, 1.0))
    .at(0.0, (property::BACKGROUND, Color::from_rgb(0.12, 0.14, 0.18)))
    .at(1.0, (property::BACKGROUND, Color::from_rgb(0.20, 0.36, 0.52)))
    .finish();
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

Use `KeyframesBuilder` to collect property snapshots at normalized offsets from
`0.0` to `1.0`, then call `finish()` to compile them into a `Keyframes` value.
The finished keyframes own a `Timing`, so duration, easing, fill mode,
direction, iterations, and playback rate stay attached to the sampled property
data.

```rust
use aura_anim_iced::{Easing, KeyframesBuilder, Timing, property};

let popup_open = KeyframesBuilder::new()
    .with_timing(Timing::new(280.0).with_easing(Easing::EaseOut))
    .at(0.0, (property::OPACITY, 0.0))
    .at(0.0, (property::SCALE, 0.92))
    .at(0.68, (property::SCALE, 1.07))
    .at(1.0, (property::OPACITY, 1.0))
    .at(1.0, (property::SCALE, 1.0))
    .finish();
```

Duplicate offsets are merged. If the same property appears multiple times at
the same offset, the later value wins.

## Timeline Orchestration

Timelines combine keyframe tracks into sequences, parallel groups, and holds.
Use sequences for lifecycle animation, parallel groups for coordinated property
changes, and holds when a state should remain visible before the next step.

```rust
use aura_anim_iced::{
    Duration, Easing, Hold, KeyframesBuilder, Timeline, Timing, Track, property,
};

let enter = Track::new(
    KeyframesBuilder::new()
        .with_timing(Timing::new(220.0).with_easing(Easing::EaseOut))
        .at(0.0, (property::OPACITY, 0.0))
        .at(1.0, (property::OPACITY, 1.0))
        .finish(),
);

let exit = Track::new(
    KeyframesBuilder::new()
        .with_timing(Timing::new(180.0).with_easing(Easing::EaseIn))
        .at(0.0, (property::OPACITY, 1.0))
        .at(1.0, (property::OPACITY, 0.0))
        .finish(),
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
use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, KeyframesBuilder, Timing, property,
};

let mut runtime = AnimationRuntime::new();
let target = AnimationTargetId::new();

let registration = runtime.register_keyframes(
    target,
    KeyframesBuilder::new()
        .with_timing(Timing::new(120.0))
        .at(0.0, (property::OPACITY, 0.0))
        .at(1.0, (property::OPACITY, 1.0))
        .finish(),
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

## Property Change Animation

Use `PropertyTransition` when an application value should animate whenever its
target changes. The first observed value seeds the stable baseline and does not
start an animation. Later different values register keyframes from the current
visual result to the new target.

`BehaviorRule` stores reusable property and timing settings. Bind it to one or
more targets to create independent transition trackers.

```rust
use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, BehaviorRule, Easing, PropertyTransition,
    Timing, WIDTH,
};

struct Panel {
    runtime: AnimationRuntime,
    target: AnimationTargetId,
    width: PropertyTransition<aura_anim_iced::property::Scalar>,
    rendered_width: f32,
}

impl Panel {
    fn new() -> Self {
        let mut runtime = AnimationRuntime::new();
        let target = AnimationTargetId::new();
        let rule = BehaviorRule::new(WIDTH)
            .with_timing(Timing::new(180.0).with_easing(Easing::EaseOut));
        let mut width = rule.bind(target);

        width.transition_to(&mut runtime, 240.0);

        Self {
            runtime,
            target,
            width,
            rendered_width: 240.0,
        }
    }

    fn set_width(&mut self, next_width: f32) {
        self.width.transition_from_visual(
            &mut self.runtime,
            self.rendered_width,
            next_width,
        );
    }
}
```

On each animation tick, merge the target snapshot into the value used by `view`
and let the transition clear its active handle when the runtime finishes:

```rust
use aura_anim_iced::{PropertyValue, WIDTH, iced_ext};

fn update_tick(panel: &mut Panel, tick: std::time::Instant) {
    let output = iced_ext::update_tick(&mut panel.runtime, tick);

    if let Some(snapshot) = output.properties_for(panel.target)
        && let Some(entry) = snapshot.find_property(&WIDTH.raw())
        && let PropertyValue::Scalar(width) = entry.value()
    {
        panel.rendered_width = *width;
    }

    panel.width.handle_completion(&panel.runtime);
}
```

The `examples/behavior_width.rs` example shows the same flow in a runnable Iced
application with controls for repeated value changes.

## State-Driven Animation

Use `StateAnimator` when the application has a small state machine and each
state pair should launch a specific timeline. A `StateTransitionSet` stores the
known pairs and can also provide a fallback timeline for unlisted changes.

```rust
use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, Duration, OPACITY, StateAnimator,
    StateTransition, StateTransitionSet, Timeline, Track,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PanelState {
    Closed,
    Open,
    Disabled,
}

fn opacity_timeline(from: f32, to: f32, ms: f64) -> Timeline {
    Timeline::track(Track::from(OPACITY, from).to(to).duration(Duration::from_millis(ms)))
}

let mut runtime = AnimationRuntime::new();
let target = AnimationTargetId::new();
let mut animator = StateAnimator::new(target, PanelState::Closed);
let transitions = StateTransitionSet::from_transitions([
    StateTransition::new(
        PanelState::Closed,
        PanelState::Open,
        opacity_timeline(0.0, 1.0, 160.0),
    ),
    StateTransition::new(
        PanelState::Open,
        PanelState::Closed,
        opacity_timeline(1.0, 0.0, 120.0),
    ),
])
.with_fallback(opacity_timeline(0.4, 1.0, 100.0));

let registration = animator.transition_to(&mut runtime, PanelState::Open, &transitions);
assert!(registration.is_some());
```

`StateAnimator::current` is updated as soon as a transition starts, while
`active_transition` and `active_progress_at` expose runtime metadata for loading
indicators, navigation locks, or diagnostics. Call `handle_completion` after
ticks when application code needs the cached active transition to match the
runtime exactly.

## Retargeting And Interruption

Retargeting is for active animations that receive a new destination. The
replacement starts from the active animation's last sampled visual value, not
from the previous target.

```rust
let mut runtime = AnimationRuntime::new();
let target = AnimationTargetId::new();
let mut opacity = PropertyTransition::new(target, aura_anim_iced::OPACITY)
    .with_timing(Timing::new(200.0));

opacity.transition_to(&mut runtime, 0.0);
opacity.transition_to(&mut runtime, 1.0);

// After one or more ticks, continue from the rendered value to the new target.
let retargeted = opacity.retarget_to(&mut runtime, 0.35);
```

Interruption is for cases where application code already knows the rendered
value, such as drag cancellation or repeated user input. It can replace an
active animation even when the destination has not changed.

```rust
let visual_opacity = 0.42;
let interrupted = opacity.interrupt_from_visual(&mut runtime, visual_opacity, 1.0);
```

Both paths cancel the superseded runtime handle after registering the
replacement. That prevents interrupted animations from later reporting
completion or overriding the replacement output.

## Route Transition Guide

Use route transitions when changing screens should animate the leaving and
entering views independently.

1. Store a `RouteAnimator<Route>` in application state.
2. Give the outgoing and incoming screen layers separate `AnimationTargetId`
   values.
3. Build a `RouteScreenTransition` from an outgoing timeline and an incoming
   timeline or `RouteIncomingMotion`.
4. Register it with `transition_screens_with`.
5. On ticks, merge snapshots for both screen targets into the effects used by
   `view`.
6. When the route or incoming handle completes, clear temporary leaving-screen
   state.

```rust
use aura_anim_iced::{
    AnimationRuntime, AnimationTargetId, Duration, OPACITY, RouteAnimator,
    RouteIncomingMotion, RouteScreenTargets, RouteScreenTransition, Timeline, Track,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Route {
    Home,
    Reports,
}

fn outgoing() -> Timeline {
    Timeline::track(Track::from(OPACITY, 1.0).to(0.0).duration(Duration::from_millis(180.0)))
}

let mut runtime = AnimationRuntime::new();
let route_target = AnimationTargetId::new();
let outgoing_target = AnimationTargetId::new();
let incoming_target = AnimationTargetId::new();
let mut animator = RouteAnimator::new(route_target, Route::Home);

let transition = RouteScreenTransition::with_incoming_motion(
    Route::Home,
    Route::Reports,
    outgoing(),
    RouteIncomingMotion::new(
        iced::Vector::new(48.0, 0.0),
        Duration::from_millis(220.0),
    ),
);

let registration = animator.transition_screens_with(
    &mut runtime,
    &transition,
    RouteScreenTargets::new(outgoing_target, incoming_target),
);

assert!(registration.is_some());
```

`RouteIncomingMotion` builds an incoming timeline that fades from `0.0` to `1.0`
and translates from the supplied offset to `iced::Vector::new(0.0, 0.0)`.
Repeated navigation replaces the active route, outgoing, and incoming handles
as a group, so stale screen animations are canceled together.

The `examples/route_transition.rs` example shows a complete Iced flow with
navigation buttons, overlaid screen cards, snapshot merging, and cleanup after
the incoming screen reaches its final state.
