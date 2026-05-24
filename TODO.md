# aura-anim-iced v0.1

## Repository Foundation

- `[x]` Create the v0.1 crate structure with `src/lib.rs`, module declarations, public re-exports, and a minimal compile path for `animatable`, `property`, `timing`, `keyframes`, `timeline`, `runtime`, and `iced_ext`.
- `[x]` Configure `Cargo.toml` with package metadata, Rust edition, initial feature flags, optional `iced` integration, dev dependencies for examples, and benchmark dependencies.
- `[x]` Add a `prelude` module that exports the v0.1 public API surface for `Animatable`, `UiProperty`, `PropertyValue`, `Timing`, `Keyframes`, `Timeline`, and `AnimationRuntime`.
- `[x]` Write crate-level documentation that states the Iced-first animation model, the v0.1 scope, the runtime integration path, and the example entry points.
- `[x]` Add compile-only smoke tests that import the public prelude, construct the core v0.1 types, and verify the crate builds with default features.

## Animatable Values

- `[x]` Implement `src/animatable.rs` with the `Animatable` trait, a clamped progress helper, and interpolation helpers for copy-friendly primitive values.
- `[x]` Implement scalar `Animatable` support for `f32`, `f64`, `i32`, and `u8`, including rounding and clamping behavior for integer outputs.
- `[x]` Implement color interpolation for the selected Iced color type, covering red, green, blue, and alpha channels with normalized progress.
- `[x]` Implement geometry interpolation for point, vector, size, and rectangle value shapes used by Iced-style UI animation.
- `[x]` Implement shadow interpolation with offset, blur, and color fields so button and popup examples can animate elevation.
- `[ ]` Add `Animatable` unit tests for scalar rounding, color midpoint sampling, geometry midpoint sampling, shadow sampling, and progress clamping.

## Property Model

- `[ ]` Implement `src/property.rs` with `UiProperty`, stable property IDs, and the core visual properties needed for opacity, transform, size, radius, color, and shadow.
- `[ ]` Implement `PropertyValue` as a typed enum for v0.1 values, including scalar, color, geometry, shadow, and transform-friendly variants.
- `[ ]` Add property-to-value matching helpers that accept valid property/value pairs and return typed errors for mismatched animation input.
- `[ ]` Add property composition ordering for opacity, transform, size, radius, background, border color, text color, and shadow.
- `[ ]` Write property tests for stable IDs, property ordering, value matching, mismatch errors, and parallel property storage.

## Timing and Easing

- `[ ]` Implement `src/timing.rs` with `Timing`, `Duration`, `Delay`, `Direction`, `FillMode`, iteration count, playback rate, and elapsed-time normalization.
- `[ ]` Add easing support that bridges to Iced easing where available and supplies v0.1 helpers for linear, ease-in, ease-out, and ease-in-out sampling.
- `[ ]` Implement fill behavior for before-start, active, after-end, forwards, backwards, and both-filled sampling states.
- `[ ]` Implement direction sampling for normal, reverse, alternate, and alternate-reverse playback across repeated iterations.
- `[ ]` Write timing tests for delay, duration normalization, fill mode output, playback rate, repeat iteration, and reverse direction sampling.

## Keyframes

- `[ ]` Implement `src/keyframes.rs` with `Keyframes<T>`, keyframe offset storage, timing attachment, builder-style `at` insertion, and sorted keyframe normalization.
- `[ ]` Implement keyframe segment lookup with edge handling for empty tracks, single-frame tracks, exact offsets, and between-frame sampling.
- `[ ]` Implement keyframe value sampling by interpolating neighboring `Animatable` values through segment progress and easing.
- `[ ]` Add multi-property keyframe support by mapping `UiProperty` to `PropertyValue` snapshots across normalized offsets.
- `[ ]` Add keyframe builder helpers for opacity, scale, translation, background color, border color, text color, and shadow.
- `[ ]` Write keyframe tests for offset normalization, segment lookup, easing application, fill mode output, and multi-property sampling.

## Timeline

- `[ ]` Implement `src/timeline.rs` with `Timeline`, `TimelineStep`, `Track`, `Sequence`, `Parallel`, `Hold`, named markers, and total-duration calculation.
- `[ ]` Implement sequence sampling that advances through ordered steps, accounts for hold segments, and returns the active property snapshot.
- `[ ]` Implement parallel sampling that merges active tracks, resolves property collisions by insertion order, and emits a composed snapshot.
- `[ ]` Implement timeline builder helpers for `sequence`, `parallel`, `hold`, `then`, `track`, `from`, `to`, `duration`, and `easing`.
- `[ ]` Add playback controls for seek, pause, resume, cancel, finish, and completion state snapshots without runtime ownership.
- `[ ]` Write timeline tests for sequence duration, parallel duration, hold sampling, property merge ordering, seek output, and completion output.

## Runtime

- `[ ]` Implement `src/runtime.rs` with `AnimationRuntime`, `AnimationRegistry`, animation handles, active entries, clock abstraction, and motion policy storage.
- `[ ]` Implement runtime registration for keyframe and timeline instances with start time, playback state, property snapshot output, and completion tracking.
- `[ ]` Implement runtime tick processing that advances active animations, removes completed entries, and returns an aggregated snapshot for view code.
- `[ ]` Implement idle detection that reports zero active animations and exposes a subscription gate for stopping animation ticks.
- `[ ]` Add testing clock support that injects deterministic timestamps for unit tests and example-level runtime checks.
- `[ ]` Write runtime tests for handle registration, tick sampling, completion removal, idle detection, and deterministic clock progression.

## Iced Integration

- `[ ]` Implement `src/iced_ext.rs` with a subscription helper that maps runtime activity into an Iced `Subscription` tick stream.
- `[ ]` Add update helper functions that route tick messages into `AnimationRuntime` and return view-friendly effect snapshots.
- `[ ]` Add effect snapshot conversion helpers for opacity, translate, scale, radius, color, and shadow values consumed by Iced widgets.
- `[ ]` Add compile checks for Iced integration behind the `iced` feature and a core-only build path with that feature disabled.
- `[ ]` Write integration tests for subscription gating, tick forwarding, active runtime updates, idle runtime output, and feature-gated compilation.

## Examples

- `[ ]` Build the `examples/animated_button.rs` demo with hover, pressed, focus, background, border, shadow, and scale animation using parallel tracks.
- `[ ]` Build the `examples/keyframes_popup.rs` demo with popup opacity, scale overshoot, settle timing, and keyframe-driven effect snapshots.
- `[ ]` Build the `examples/timeline_toast.rs` demo with enter, hold, exit, opacity, translate-y, and completion cleanup through the runtime.
- `[ ]` Add shared example helpers for app state, runtime storage, tick messages, and minimal reusable animated style mapping.
- `[ ]` Add example README snippets that show the run commands and the purpose of each v0.1 example.

## Testing and Benchmarks

- `[ ]` Add unit test modules for `animatable`, `property`, `timing`, `keyframes`, `timeline`, `runtime`, and `iced_ext`.
- `[ ]` Add integration tests that run keyframes, timeline sequence, timeline parallel, runtime tick, and idle subscription behavior together.
- `[ ]` Configure benchmark targets for 100, 1,000, and 10,000 property track samples with zero-allocation sampling assertions.
- `[ ]` Add benchmark fixtures for scalar tracks, color tracks, geometry tracks, shadow tracks, and mixed-property timeline snapshots.
- `[ ]` Add CI-friendly commands for `cargo fmt`, `cargo clippy`, `cargo test`, `cargo check --examples`, and benchmark compilation.

## Documentation

- `[ ]` Update `README.md` with the project positioning, the relationship to Iced `Animation<T>`, installation commands, and a minimal runtime example.
- `[ ]` Add README sections for animatable values, property tracks, keyframes, timeline orchestration, runtime ticking, and Iced subscription wiring.
- `[ ]` Add docs.rs examples to public types showing one compact usage snippet per core v0.1 module.
- `[ ]` Add `CHANGELOG.md` with the `0.1.0-alpha.1` scope, implemented modules, example names, and benchmark entry points.
- `[ ]` Add `LICENSE`, repository metadata, badges, and documentation links required for a clean crates.io package page.

## Release

- `[ ]` Run the v0.1 release gate with formatting, linting, tests, example checks, docs generation, and benchmark compilation.
- `[ ]` Package the crate with `cargo package` and inspect the generated archive contents for source files, examples, README, changelog, and license.
- `[ ]` Publish `0.1.0-alpha.1` to crates.io after the release gate passes and the package archive contains the documented v0.1 scope.
- `[ ]` Tag the repository with `v0.1.0-alpha.1` and write release notes covering Animatable, PropertyValue, Timing, Keyframes, Timeline, Runtime, Iced integration, examples, tests, and benchmarks.
