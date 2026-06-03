# Changelog

## Unreleased

### Added

- Added `AnimationFlow` as a standard Iced integration path that owns a runtime,
  reuses tick output, captures registration-time visual values, routes tick
  updates, and exposes subscription gating from one product-facing API.
- Added `AnimationTargetOutput` for reusable target-scoped visual output reads
  from product view code.
- Added shared completion cleanup through `AnimationCompletionCleanup` and `AnimationFlow::cleanup_completed`.
- Added `DefaultMotions` for product-level duration, easing, fill mode,
  color interpolation, and spring motion defaults.
- Added a compact product quick start covering value animation, state
  animation, widget motion, theme switching, and spring motion defaults.
- Added feature flags for palette, spring, widgets, theme, layout, serde,
  tracing, and inspector work while keeping the core animation API available
  without default features.
- Added feature-gated product defaults, widget-oriented example declarations,
  documentation snippets, and tests for optional product extension layers.
- Changed the crate root to expose public modules without root-level
  re-exports, and reduced the prelude to common product animation imports.

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
