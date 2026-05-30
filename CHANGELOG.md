# Changelog

## Unreleased

### Added

- Added `PropertyTransition` for automatically registering property animations when a tracked visual value changes.

## v0.1.1

### Performance Optimizations

- Compiled `KeyframesBuilder` snapshots into sorted per-property tracks during `finish()`, so sampling no longer scans every frame for every property.
- Added `KeyframesBuilder` to the crate root and prelude as the public construction API for finished `Keyframes`.
- Updated timeline property builders so `Track::from(...).to(...).duration(...)` remains a builder chain and finishes explicitly or through `Into<Track>`.
