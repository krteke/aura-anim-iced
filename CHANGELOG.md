# Changelog

## Unreleased

### Added

- Added `PropertyTransition` for automatically registering property animations when a tracked visual value changes.
- Added `BehaviorRule` so value change animation settings can be reused across multiple targets.
- Added smooth continuation for property transitions that receive a new target while a previous transition is still running.
- Added an explicit visual-value transition path so property changes can start from the value currently rendered on screen.
- Added completion handling for property transitions.
- Added `StateAnimator` and `StateTransition` for registering timelines when application state changes.
- Added `StateTransitionSet` so state changes can match and launch the correct transition timeline.
- Added fallback timelines for state switches that do not have a custom transition.
- Added active state transition progress tracking with elapsed time and normalized progress.
- Added state transition active-cache refresh so completed or canceled runtime handles are cleared automatically before starting another state transition, with manual refresh available when exact active metadata is needed.
- Added `PropertyTransition::retarget_to` for changing the destination of a running property animation from its current visual value.
- Added `PropertyTransition::interrupt_from_visual` for replacing interrupted animations from the value currently rendered on screen.

## v0.1.1

### Performance Optimizations

- Compiled `KeyframesBuilder` snapshots into sorted per-property tracks during `finish()`, so sampling no longer scans every frame for every property.
- Added `KeyframesBuilder` to the crate root and prelude as the public construction API for finished `Keyframes`.
- Updated timeline property builders so `Track::from(...).to(...).duration(...)` remains a builder chain and finishes explicitly or through `Into<Track>`.
