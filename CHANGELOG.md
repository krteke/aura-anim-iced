# Changelog

## Unrelease

### Added

- Spring channels for independent value interpolation on struct fields.
- Added reusable `MotionBinding<S, T>` configuration for mapping business
  states to target values and transition factories.
- Added per-consumer `MotionBindingState<S>` tracking so one binding
  configuration can drive multiple independent controls.
- Added `TransitionContext<S, T>` with the previous and next business states,
  current sampled value, and resolved target value.
- Added exact state-pair transition factories and fallback factories supporting
  Tween, Spring, Keyframes, Timeline, and custom animation sources.
- Added `BoxAnimation<T>` and `AnimationExt::boxed()` for type-erased
  transition factory results.
- Added `SpringConfig::snappy()` as a responsive control-animation preset.
- Added an optional `tracing` feature, forwarded by the core, Iced, and facade
  crates, with diagnostics for runtime lifecycle operations, invalid handles,
  motion bindings, and Presence state changes.

### Breaking Changes

- Replaced ambiguous `Option`/`bool` motion access and mutation results with
  `Result<_, MotionError>`. Runtime failures now distinguish out-of-bounds
  slots, removed animations, stale generations, and value type mismatches.
- Changed `Motion::value` and `Motion::value_ref` to return errors instead of
  panicking for invalid handles.
- Changed Presence playback, value access, and synchronization methods to
  propagate `MotionError` instead of silently ignoring invalid motions.

### Changed

- Completed retained animations now move their sampled value into inline
  runtime storage, avoiding the previous `Box<Settled<T>>` allocation and
  value clone for built-in animation sources.
- Slot generations now advance when storage is reused rather than when an
  animation is removed. Removed handles report `Removed` until reuse and
  `StaleHandle` afterward.
- Motion bindings now resolve targets, sample the current motion value,
  construct the configured animation, and call `motion.play(...)`
  automatically when business state changes.
- Binding state is committed only after playback succeeds; missing targets,
  missing transition factories, and stale motion handles return explicit
  `MotionBindingError` values.
- Updated the interactive button example to use a shared `MotionBinding`
  configuration for Tween hover and Spring press transitions.

### Testing

- Added unit and public API coverage for exact and fallback transitions,
  interrupted sampled values, reusable binding configurations, error handling,
  boxed Keyframes and Timeline factories, facade exports, and the snappy spring
  preset.

## v0.2.2 - 2026-06-08

This release replaces the previous property/target registration architecture
with a typed motion runtime. It is a breaking architecture reset while the
project is still pre-1.0.

### Breaking Changes

- Replaced `AnimationRuntime`, `AnimationTargetId`, property snapshots, property
  tracks, behavior bindings, and target-based lookup with `MotionRuntime` and
  typed `Motion<T>` handles.
- Applications now store `Motion<T>` handles and read values with
  `motion.value(&runtime)` instead of querying runtime targets or properties.
- Animation objects are owned and ticked by `MotionRuntime`; application code no
  longer stores or manually ticks individual tween, spring, or keyframe objects.
- Replaced property-oriented timelines with value-oriented
  `Sequence<T>`, `Parallel<T>`, and `Hold<T>` animation sources.
- Reworked crate exports and preludes around the new typed runtime. Existing
  code using the v0.2.0/v0.2.1 property, behavior, state, route, or effect APIs
  must migrate to typed motion values.

### Added

- Added `Motion<T>`, a lightweight typed handle containing a slot ID,
  generation, and type marker.
- Added `MotionRuntime` with typed motion creation, centralized ticking,
  retargeting, playback commands, value access, slot removal, and capacity
  inspection.
- Added `#[derive(Animatable)]` for named, tuple, and unit structs whose fields
  implement `Animatable`.
- Added the common `Animation<T>` protocol with value, state, duration,
  overflow-aware advancement, pause, resume, cancel, seek, finish, and optional
  retargeting.
- Added value-oriented `Tween<T>`, `Keyframes<T>`, and `Spring<T>` animation
  sources.
- Added recursive timeline composition through `Sequence<T>`, `Parallel<T>`,
  and `Hold<T>`. Nested forms such as
  `Sequence(Parallel(Sequence, Sequence), Sequence)` are supported.
- Added explicit parallel compositors so independent branches can animate
  different fields without implicit last-writer behavior.
- Added `Presence<T>` for enter/exit motion while keeping content mounted until
  its exit animation completes.
- Added `RetainPolicy` and `MotionRuntime::play_once` for transient animations
  that automatically release their slot after completion or cancellation.
- Added interpolation implementations for scalar integers, floats, tuples, and
  arrays.
- Added an `aura-anim` facade crate that re-exports the core runtime, Iced
  integration, and common prelude.

### Iced Integration

- Added `Interpolate` support for `iced::Vector`, `Point`, `Size`, `Rectangle`,
  `Padding`, and `border::Radius`.
- Added color-feature-gated interpolation for `iced::Color`, `Shadow`, and
  `Border`.
- Added the default `rgba` feature for component-wise sRGB-alpha interpolation.
- Added the mutually exclusive `oklaba` feature for Oklab RGB interpolation
  with independently interpolated alpha.
- Added `TickPolicy::Frames`, `TickPolicy::interval`, and `TickPolicy::fps`.
- Added `subscription_with_policy`; Iced frame or timer subscriptions are only
  active while the runtime contains active animations.
- Added `frame` integration using elapsed wall-clock time through
  `MotionRuntime::tick_at`.

### Changed

- `Tween::transition_to` and runtime retargeting now continue from the currently
  sampled value instead of restarting from a stale origin.
- `Animation::advance` returns unconsumed time so sequences can cross multiple
  children correctly during large frame deltas.
- Parallel duration is determined by the longest finite branch, while sequence
  duration is the sum of finite child durations.
- Parallel seek maps global timeline progress to each child using that child's
  duration.
- Keyframes support per-segment easing, duplicate-time replacement, delays,
  playback direction, and finite or infinite iteration counts.
- Spring interpolation supports overshoot, seek, finish, and live retargeting.
- Split the workspace into `aura-anim-core`, `aura-anim-iced`,
  `aura-anim-macros`, and the `aura-anim` facade.
- Unified workspace package metadata, internal dependency versions, docs.rs
  feature selection, and release packaging at version `0.2.2`.

### Lifecycle And Performance

- Runtime ticks only active slots instead of scanning all retained motions.
- Added an active queue with duplicate-queue prevention and O(1)
  `has_active`/`active_count` checks.
- Completed retained animations are compacted to a lightweight settled value,
  releasing keyframe and timeline trees while keeping the typed handle valid.
- Removed and transient slots are reused through a free list.
- Added generation checks so stale handles cannot access animations allocated
  into reused slots.
- Added `motion_count`, `slot_capacity`, and `shrink_to_fit` runtime inspection
  and capacity controls.
- Avoided type downcasts in the runtime tick loop; downcasting is limited to
  typed value access and retargeting.

### Examples

- Added focused Iced examples for tween interruption, keyframes, spring
  retargeting, timeline composition, interactive buttons, menus,
  notifications, and route transitions.
- Added an interactive showcase combining typed motion, Presence, keyframes,
  spring, nested sequence/parallel composition, route transitions, Iced value
  interpolation, and configurable tick policy.
- Added a command-line runtime example covering retargeting, infinite
  keyframes, spring, nested timelines, and transient slot cleanup.

### Testing And Benchmarks

- Added public API integration coverage for runtime lifecycle, retargeting,
  playback commands, keyframes, recursive timelines, Presence, Spring,
  interpolation, retain policies, and facade exports.
- Added Iced integration coverage for common Iced value interpolation, RGBA and
  Oklaba color paths, and tick policies.
- Added Criterion benchmarks for interpolation, timing, animation sources,
  nested timelines, runtime ticking, commands, slot reuse, and lifecycle
  operations.

## v0.2.1 - 2026-06-02

### Performance Optimizations

- Used `u64` instead of `uuid` for animation target IDs.
- Routed runtime sampling through the zero-distribution path.
- Changed the two-point animation in `PropertyTransition` to use a dedicated source.
- Changed `StateTransitionSet` to index and Arc.

## v0.2.0 - 2026-06-01

### Added

- Added property-change animation with `PropertyTransition` and reusable `BehaviorRule` settings.
- Added smooth continuation for property transitions so active animations can move from the current visual result to a new target.
- Added explicit visual-value transition APIs for application-owned rendered values.
- Added `PropertyTransition::retarget_to` and `PropertyTransition::interrupt_from_visual` for active animation replacement.
- Added property transition registration, active metadata, progress tracking, completion handling, and replacement cleanup.
- Added state-driven animation with `StateAnimator`, `StateTransition`, and `StateTransitionSet`.
- Added exact state-pair matching, fallback timelines, active state progress, completion handling, stale-cache refresh, and replacement reporting.
- Added route transition primitives built on the state transition system.
- Added screen-to-screen route transitions with separate outgoing and incoming targets.
- Added `RouteIncomingMotion` for built-in incoming opacity and translation motion.
- Added active route screen transition tracking and grouped cleanup for repeated navigation actions.

### Examples

- Added a runnable width behavior example driven by `BehaviorRule` and `PropertyTransition`.
- Added width behavior controls, current/target width visualization, and plain-language behavior text.
- Added a runnable route transition example that demonstrates repeated screen switching.

### Documentation

- Added README guides for property-change animation, state-driven animation, retargeting and interruption, and route transitions.
- Updated the README and crate overview to describe the v0.2 behavior, state, and route animation capabilities.
- Added rustdoc examples for the v0.2 behavior, state, and route animation modules.

### Tests

- Added regression coverage for automatic property transitions after tracked value changes.
- Added regression coverage for property retargeting from the current visual result during active animation.
- Added regression coverage for repeated state-change interruption and replacement cleanup.
- Added regression coverage for matching multiple state transition pairs to distinct timelines.
- Added regression coverage for route screen transition progress from outgoing to incoming screens.

### Performance Optimizations

- Optimized `AnimationRegistry`.

## v0.1.1

### Performance Optimizations

- Compiled `KeyframesBuilder` snapshots into sorted per-property tracks during `finish()`, so sampling no longer scans every frame for every property.
- Added `KeyframesBuilder` to the crate root and prelude as the public construction API for finished `Keyframes`.
- Updated timeline property builders so `Track::from(...).to(...).duration(...)` remains a builder chain and finishes explicitly or through `Into<Track>`.
