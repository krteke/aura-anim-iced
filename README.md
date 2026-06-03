# aura-anim-iced

Iced-first animation orchestration for applications that need coordinated
property changes, state transitions, and screen-to-screen route motion.

This crate builds on Iced's public animation surface instead of replacing it.
User-facing APIs use Iced types such as `iced::Vector`,
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

v0.2.1

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

Enable product extension features only when the application needs that layer:

```sh
cargo add aura-anim-iced --features palette
cargo add aura-anim-iced --features spring
cargo add aura-anim-iced --features widgets
```

The same configuration can be written directly in `Cargo.toml`:

```toml
[dependencies]
aura-anim-iced = "0.2.1"
```

```toml
[dependencies]
aura-anim-iced = { version = "0.2.1", features = ["inspector"] }
```

```toml
[dependencies]
aura-anim-iced = { version = "0.2.1", features = ["palette", "spring", "widgets"] }
```

## Minimal Runtime Example

Store an `AnimationRuntime` in application state, register keyframes in `update`,
keep an Iced tick subscription active while the runtime is playing,
and convert tick output into view effects for one target.

```rust
use std::time::Instant;

use aura_anim_iced::{
    iced_ext::{self, EffectSnapshot, tick_effect_snapshot_for},
    keyframes::KeyframesBuilder,
    runtime::{AnimationRuntime, AnimationTargetId},
    timing::Timing,
};

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

## Product Quick Start

Use `AnimationFlow` when product code should route value animation, state
animation, widget motion, theme switching, and future spring motion through the
same update, subscription, and view-output path.

```rust
use std::time::Instant;

use aura_anim_iced::{
    behavior::{BehaviorRule, PropertyTransition},
    color::AnimColor,
    defaults::DefaultMotions,
    iced_ext::{AnimationFlow, EffectSnapshot},
    keyframes::KeyframesBuilder,
    property::{self, BACKGROUND, OPACITY, TEXT_COLOR, WIDTH},
    runtime::{AnimationRuntime, AnimationTargetId},
    state::{StateAnimator, StateTransition, StateTransitionSet},
    timeline::{Timeline, Track},
    timing::Duration,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PanelState {
    Hidden,
    Visible,
}

struct ProductUi {
    flow: AnimationFlow<aura_anim_iced::runtime::TestClock>,
    defaults: DefaultMotions,
    panel_target: AnimationTargetId,
    card_target: AnimationTargetId,
    theme_target: AnimationTargetId,
    width: PropertyTransition<aura_anim_iced::property::Scalar>,
    panel: StateAnimator<PanelState>,
    panel_transitions: StateTransitionSet<PanelState>,
    rendered_width: f32,
    dark_theme: bool,
}

#[derive(Debug, Clone)]
enum Message {
    ResizePanel(f32),
    ShowPanel,
    ToggleTheme,
    AnimationTick(Instant),
}

impl ProductUi {
    fn new() -> Self {
        let defaults = DefaultMotions::default();
        let panel_target = AnimationTargetId::new();
        let card_target = AnimationTargetId::new();
        let theme_target = AnimationTargetId::new();
        let width = defaults.behavior(WIDTH).bind(card_target);
        let panel = StateAnimator::new(panel_target, PanelState::Hidden);
        let panel_transitions = StateTransitionSet::from_transitions([
            StateTransition::new(
                PanelState::Hidden,
                PanelState::Visible,
                Timeline::track(
                    Track::from(OPACITY, 0.0)
                        .to(1.0)
                        .duration(defaults.duration()),
                ),
            ),
        ]);

        Self {
            flow: AnimationFlow::with_runtime(AnimationRuntime::testing()),
            defaults,
            panel_target,
            card_target,
            theme_target,
            width,
            panel,
            panel_transitions,
            rendered_width: 240.0,
            dark_theme: false,
        }
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ResizePanel(next_width) => {
                if let Some(registration) = self.width.transition_from_visual(
                    self.flow.runtime_mut(),
                    self.rendered_width,
                    next_width,
                ) {
                    self.flow.capture(&registration);
                }
            }
            Message::ShowPanel => {
                if let Some(registration) = self.panel.transition_to(
                    self.flow.runtime_mut(),
                    PanelState::Visible,
                    &self.panel_transitions,
                ) {
                    self.flow.capture(&registration);
                }
            }
            Message::ToggleTheme => {
                self.dark_theme = !self.dark_theme;
                let background = if self.dark_theme {
                    iced::Color::from_rgb(0.08, 0.10, 0.14)
                } else {
                    iced::Color::WHITE
                };
                let text = if self.dark_theme {
                    iced::Color::WHITE
                } else {
                    iced::Color::BLACK
                };
                let registration = self.flow.runtime_mut().register_keyframes(
                    self.theme_target,
                    KeyframesBuilder::new()
                        .with_timing(self.defaults.timing())
                        .at(0.0, (BACKGROUND, AnimColor::from(iced::Color::TRANSPARENT)))
                        .at(1.0, (BACKGROUND, AnimColor::from(background)))
                        .at(1.0, (TEXT_COLOR, AnimColor::from(text)))
                        .finish(),
                );
                self.flow.capture(&registration);
            }
            Message::AnimationTick(tick) => {
                self.flow.update_tick(tick);
                if let Some(width) = self.flow.target(self.card_target).get(WIDTH) {
                    self.rendered_width = width;
                }
                self.flow.cleanup_completed(&mut self.width);
                self.flow.cleanup_completed(&mut self.panel);
            }
        }
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        self.flow.subscription(Message::AnimationTick)
    }

    fn card_effects(&self) -> EffectSnapshot {
        self.flow.target(self.card_target).effects()
    }

    fn panel_effects(&self) -> EffectSnapshot {
        self.flow.target(self.panel_target).effects()
    }

    fn theme_effects(&self) -> EffectSnapshot {
        self.flow.target(self.theme_target).effects()
    }

    #[cfg(feature = "spring")]
    fn spring_defaults(&self) -> aura_anim_iced::defaults::SpringMotionDefaults {
        self.defaults.spring()
    }
}
```

Widget motion is represented by ordinary target-scoped opacity, size, color,
and transform properties. Theme switching uses the same runtime path with color
properties. Dedicated widget, theme, palette, layout, inspector, and spring
helpers are compiled behind their matching feature flags. `DefaultMotions::spring()`
stores the spring feel that future spring animation sources should use when the
`spring` feature is enabled.

## Animatable Values

Public animation inputs use Iced value types wherever possible. The value model
covers scalar values, `iced::Vector`, `iced::Size`, `iced::Rectangle`,
`iced::Shadow`, transform-friendly values, and `AnimColor`. Color animation uses
`AnimColor` so the sampled value carries its color-space semantics instead of
passing interpolation mode through the runtime.

```rust
use aura_anim_iced::{color::AnimColor, keyframes::KeyframesBuilder, property, timing::Timing};
use iced::Color;

let fade_and_color = KeyframesBuilder::new()
    .with_timing(Timing::new(160.0))
    .at(0.0, (property::OPACITY, 0.0))
    .at(1.0, (property::OPACITY, 1.0))
    .at(0.0, (property::BACKGROUND, AnimColor::from(Color::from_rgb(0.12, 0.14, 0.18))))
    .at(1.0, (property::BACKGROUND, AnimColor::from(Color::from_rgb(0.20, 0.36, 0.52))))
    .finish();
```

## Property Tracks

Properties are identified by typed `PropertySpec` values. Built-in specs cover
opacity, translation, scale, size, padding, radius, colors, and shadow.
Applications can also define custom specs when an example or widget needs an
extra sampled value, such as a toast offset.

```rust
use aura_anim_iced::{
    property::{self, PropertyKey, PropertySpec},
};

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
use aura_anim_iced::{keyframes::KeyframesBuilder, property, timing::{Easing, Timing}};

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

## Palette Color Interpolation

Enable the `palette` feature when color-heavy theme or brand motion should carry
colors in perceptual Oklab space instead of sRGB. The perceptual color-space
conversion is provided by the `palette` crate.

```rust
use aura_anim_iced::{
    color::AnimColor,
    keyframes::KeyframesBuilder,
    property,
};

let theme_shift = KeyframesBuilder::new()
    .at(0.0, (
        property::BACKGROUND,
        AnimColor::oklaba_from_srgba(0.95, 0.12, 0.08, 1.0),
    ))
    .at(1.0, (
        property::BACKGROUND,
        AnimColor::oklaba_from_srgba(0.05, 0.28, 0.96, 1.0),
    ))
    .finish();
```

Without the `palette` feature, `AnimColor::Srgba` remains available and color
properties interpolate in sRGB component space.

## Timeline Orchestration

Timelines combine keyframe tracks into sequences, parallel groups, and holds.
Use sequences for lifecycle animation, parallel groups for coordinated property
changes, and holds when a state should remain visible before the next step.

```rust
use aura_anim_iced::{
    keyframes::KeyframesBuilder,
    property,
    timeline::{Hold, Timeline, Track},
    timing::{Duration, Easing, Timing},
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
    keyframes::KeyframesBuilder,
    property,
    runtime::{AnimationRuntime, AnimationTargetId},
    timing::Timing,
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

use aura_anim_iced::{iced_ext, runtime::AnimationRuntime};

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
    behavior::{BehaviorRule, PropertyTransition},
    property::WIDTH,
    runtime::{AnimationRuntime, AnimationTargetId},
    timing::{Easing, Timing},
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
use aura_anim_iced::{iced_ext, property::{PropertyValue, WIDTH}};

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
    property::OPACITY,
    runtime::{AnimationRuntime, AnimationTargetId},
    state::{StateAnimator, StateTransition, StateTransitionSet},
    timeline::{Timeline, Track},
    timing::Duration,
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
let mut opacity = PropertyTransition::new(target, aura_anim_iced::property::OPACITY)
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
    property::OPACITY,
    route::{RouteAnimator, RouteIncomingMotion, RouteScreenTargets, RouteScreenTransition},
    runtime::{AnimationRuntime, AnimationTargetId},
    timeline::{Timeline, Track},
    timing::Duration,
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
