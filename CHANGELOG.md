# Changelog

## Unreleased

### Added

- Added `PropertyTransition` for automatically registering property animations when a tracked visual value changes.
- Added `BehaviorRule` so value change animation settings can be reused across multiple targets.
- Added smooth continuation for property transitions that receive a new target while a previous transition is still running.
- Added an explicit visual-value transition path so property changes can start from the value currently rendered on screen.

## v0.1.1

### Performance Optimizations

- Compiled `KeyframesBuilder` snapshots into sorted per-property tracks during `finish()`, so sampling no longer scans every frame for every property.
- Added `KeyframesBuilder` to the crate root and prelude as the public construction API for finished `Keyframes`.
- Updated timeline property builders so `Track::from(...).to(...).duration(...)` remains a builder chain and finishes explicitly or through `Into<Track>`.
