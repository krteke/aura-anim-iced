# Changelog

## Unreleased

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

### Changed

- Split the property module by responsibility while preserving the public property API.
- Made Iced a core dependency because the crate targets advanced animation for Iced applications.
- Reframed pure calculation modules as internal support for Iced-first animation orchestration.
- Exposed Iced interpolation modules and Iced property value variants unconditionally.
- Removed internal interpolation traits from the public prelude so user-facing APIs stay centered on Iced property snapshots.
- Changed keyframe placeholders from generic single-value animation storage to sampled Iced property snapshots.
- Split timing internals by duration, iteration, mode, normalization, sampling, and utility responsibilities without changing existing timing builders.
