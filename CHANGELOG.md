# Changelog

## Unreleased

### Added

- Rebuilt the v0.1 crate skeleton with core module declarations, public re-exports, and minimal compile-path placeholder types.
- Replaced the package manifest with v0.1 metadata, feature flags, optional Iced integration, and initial development dependencies.
- Added the v0.1 prelude module for the core public API surface.
- Added crate-level documentation for the Iced-first model, v0.1 scope, runtime integration path, and planned examples.
- Added a compile-only prelude smoke test for the default feature build path.
- Added core interpolation helpers for clamped progress and copy-friendly primitive sampling.

### Changed

- Aligned optional Iced integration with Iced's default feature set instead of maintaining a partial native feature selection.
