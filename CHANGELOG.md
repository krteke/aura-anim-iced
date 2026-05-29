# Changelog

## Unreleased

### Release Scope

- Prepared `0.1.1-alpha.1` as an Iced-first animation foundation release for typed property snapshots, timing, keyframes, timeline orchestration, runtime ticking, and Iced integration helpers.
- Kept higher-level behavior rules, widget wrappers, gesture drivers, layout transitions, theme transitions, and visual inspector UI outside this alpha scope.

### Implemented Modules

- `property`: typed property specs, sampled property values, composition ordering, property snapshots, and mismatch validation.
- `timing`: duration, delay, direction, fill mode, iteration count, playback rate, easing, and elapsed-time normalization.
- `keyframes`: normalized keyframe storage, duplicate-offset merging, per-property interpolation, and keyframe sampling.
- `timeline`: tracks, sequences, parallel groups, holds, total-duration calculation, timeline sampling, and playback controls.
- `runtime`: animation handles, target-scoped registration, active entry storage, deterministic test clocks, tick aggregation, completion reporting, and idle detection.
- `iced_ext`: Iced subscription gating, runtime tick forwarding, and `EffectSnapshot` conversion for view code.
- `prelude`: compact imports for the v0.1 public API surface.

### Examples

- `animated_button`: hover, press, focus, background, border, text color, shadow, radius, and scale animation using parallel timeline tracks.
- `keyframes_popup`: popup open and close animation using opacity, scale overshoot, radius, color, shadow, and runtime completion cleanup.
- `timeline_toast`: enter, hold, exit, opacity, scale, shadow, custom toast offset, and cleanup using a timeline sequence.

### Benchmarks

- `sampling`: Criterion benchmark target for `keyframes/sample_counts`, `keyframes/value_fixtures`, and `timeline/mixed_snapshots`.
- `keyframes/sample_counts`: samples scalar keyframes at 100, 1,000, and 10,000 iterations.
- `keyframes/value_fixtures`: compares scalar, color, geometry, and shadow keyframe sampling fixtures.
- `timeline/mixed_snapshots`: samples mixed-property timeline snapshots at 100, 1,000, and 10,000 iterations.

### Added

- Rebuilt the v0.1 crate skeleton with module declarations, public re-exports, and minimal compile-path placeholder types.
- Replaced the package manifest with v0.1 metadata, feature flags, a core Iced dependency, and initial development dependencies.
- Added the v0.1 prelude module for the Iced-first public API surface.
- Added crate-level documentation for the Iced-first model, Iced public API boundary, v0.1 scope, runtime integration path, and planned examples.
- Added a compile-only prelude smoke test for the default feature build path.
- Added core interpolation helpers for clamped progress and copy-friendly primitive sampling.
- Added internal scalar interpolation coverage for floating-point and integer animation support.
- Added `iced::Color` interpolation across red, green, blue, and alpha channels.
- Added Iced geometry interpolation for points, vectors, sizes, and rectangles.
- Added shadow interpolation for Iced shadows.
- Added internal interpolation tests for scalar rounding, progress clamping, color, geometry, and shadow sampling.
- Added stable UI property identifiers, visual categories, and default composition order metadata.
- Added typed property values for scalar values, Iced geometry, transform data, Iced colors, and Iced shadows.
- Added property/value validation helpers with typed mismatch errors.
- Added deterministic property composition keys and sorting helpers for applying property snapshots in visual order.
- Added property model tests for stable IDs, composition order, value validation, mismatch diagnostics, and parallel property storage.
- Added structured timing primitives for duration, delay, direction, fill mode, iteration count, playback rate, and elapsed-time normalization.
- Added timing easing support using `iced::animation::Easing` directly.
- Added `PropertySnapshot` as the public sampled-property container used by keyframes and timelines.
- Added fill-mode-aware timing sample states for active samples, skipped before/after intervals, backwards fill, forwards fill, and both-filled timing.
- Added direction-aware timing sampling for normal, reverse, alternate, and alternate-reverse playback across repeated iterations and fill endpoints.
- Added timing regression tests for delay boundaries, zero-duration completion, fill output, playback rate scaling, repeated iteration completion, and reverse playback.
- Added property-snapshot keyframe storage with normalized offsets, timing attachment, builder-style insertion, and sorted snapshot composition.
- Added keyframe segment lookup for empty tracks, single-frame tracks, exact offsets, edge offsets, and between-frame progress.
- Added keyframe snapshot sampling with Iced easing and interpolation for scalar, Iced geometry, transform, color, and shadow values.
- Added batch keyframe insertion APIs with single-pass normalization and duplicate-offset merging.
- Added per-property keyframe sampling so each `UiProperty` maps across its own normalized offsets in multi-property snapshots.
- Added keyframe builder helpers for common opacity, scale, translation, color, and shadow tracks.
- Added keyframe regression tests for normalization, segment lookup, easing, fill behavior, and multi-property sampling.
- Added timeline structure primitives for tracks, sequences, parallel groups, holds, markers, and total duration calculation.
- Added sequence timeline sampling with ordered step advancement, hold gaps, and active track snapshots.
- Added parallel timeline sampling with active track merging and insertion-order collision resolution.
- Added timeline builder helpers for sequence, parallel, hold, track, chained steps, and track keyframe timing.
- Added runtime-independent timeline playback controls for seeking, pausing, resuming, canceling, finishing, and completion snapshots.
- Added timeline regression coverage for duration, hold sampling, property merge order, seek output, and completion output.
- Added runtime storage primitives for animation handles, active entries, registries, clocks, and motion policy configuration.
- Added timeline marker helpers for name lookup, offset-ordered storage, stable same-offset ordering, and offset-based filtering.
- Added runtime registration for keyframes and timelines with start timestamps, initial snapshots, and completion tracking.
- Added runtime tick processing that advances active animations, merges property snapshots, emits completion output, and removes completed entries.
- Added runtime idle detection and a subscription gate for stopping animation ticks when no playing animations remain.
- Added a deterministic runtime test clock for unit tests and example-level animation checks.
- Added runtime regression coverage for handle registration, tick sampling, completion removal, idle detection, and deterministic clock progression.
- Added an Iced subscription helper that maps active runtime animations into a tick stream.
- Added an Iced update helper that routes tick messages into the runtime and returns sampled animation output.
- Added effect snapshot conversion helpers for applying sampled opacity, translation, scale, radius, colors, and shadow values to Iced widgets.
- Added compile checks for Iced integration helpers, example usage, and public prelude imports.
- Added integration coverage for subscription gating, tick forwarding, active runtime updates, and idle runtime output.
- Added an animated button example covering hover, pressed, focus, background, border, shadow, and scale animation with parallel tracks.
- Added real v0.1 feature flags for default builds, serde, tracing, inspector diagnostics, and testing.
- Added feature-gated runtime diagnostics through `tracing`, with inspector-specific tick events for tooling builds.
- Added README guidance for project positioning, installation, Iced `Animation<T>` boundaries, and minimal runtime integration.
- Added README sections for animatable values, property tracks, keyframes, timeline orchestration, runtime ticking, and Iced subscription wiring.
- Added docs.rs examples for property snapshots, timing, keyframes, timelines, runtime registration, Iced effects, subscriptions, and the public prelude.

### Changed

- Split the property module by responsibility while preserving the public property API.
- Made Iced a core dependency because the crate targets advanced animation for Iced applications.
- Reframed pure calculation modules as internal support for Iced-first animation orchestration.
- Exposed Iced interpolation modules and Iced property value variants unconditionally.
- Removed internal interpolation traits from the public prelude so user-facing APIs stay centered on Iced property snapshots.
- Changed keyframe placeholders from generic single-value animation storage to sampled Iced property snapshots.
- Split timing internals by duration, iteration, mode, normalization, sampling, and utility responsibilities without changing existing timing builders.
- Split keyframe storage, segment lookup, track operations, and tests into focused modules.
- Defined duplicate keyframe offsets as merged snapshots where later values override earlier values for the same property.
- Split timeline internals into focused modules while preserving the public timeline API.
