# aura-anim-iced v0.1 TODO

## Release Target

- `[ ]` Implement easing curves for UI animation.
- `[ ]` Implement type interpolation for core scalar and UI value types.
- `[ ]` Implement `Tween<T>` for single-segment animation.
- `[ ]` Implement `Transition<T>` for state-driven animation.
- `[ ]` Implement animation clock helpers.
- `[ ]` Implement Iced subscription helpers.
- `[ ]` Create three runnable examples.
- `[ ]` Write unit tests for core boundary behavior.
- `[ ]` Write public API documentation for docs.rs.
- `[ ]` Package the crate for crates.io.

## Project Shell

- `[x]` Configure feature flags: `iced`, `std`, `serde`, `tracing`, `testing`, `widgets`, and `spring`.
- `[x]` Configure optional dependencies for `iced`, `serde`, and `tracing`.
- `[x]` Configure Iced `0.14` as the initial target version.
- `[x]` Configure Rust `1.88` as the MSRV.
- `[x]` Configure Rust lint rules.
- `[x]` Configure Clippy lint rules.
- `[x]` Configure rustfmt.
- `[x]` Create the CI workflow.
- `[x]` Create the release workflow.
- `[x]` Create the changelog file.

## API Specification

- `[ ]` Write `docs/api-v0.1.md` with final public names for `Easing`, `Tween`, `Transition`, and clock helpers.
- `[ ]` Write `docs/api-v0.1.md` entries for duration helper functions.
- `[ ]` Write `docs/api-v0.1.md` entries for `Tween::sample`.
- `[ ]` Write `docs/api-v0.1.md` entries for `Transition::set_target`.
- `[ ]` Write `docs/api-v0.1.md` entries for `Transition::value_at`.
- `[ ]` Write `docs/api-v0.1.md` entries for transition retarget behavior.
- `[ ]` Write `docs/api-v0.1.md` entries for zero-duration tween behavior.
- `[ ]` Write `docs/api-v0.1.md` entries for zero-duration transition behavior.
- `[ ]` Write `docs/api-v0.1.md` entries for delay behavior.
- `[ ]` Write `docs/api-v0.1.md` entries for completion queries.
- `[ ]` Write `docs/api-v0.1.md` entries for Iced subscription helper names.
- `[ ]` Add crate-level documentation links to `docs/api-v0.1.md`.

## `easing`

- `[x]` Create the public `Easing` type in `src/easing.rs`.
- `[x]` Implement normalized progress clamping in `src/easing.rs`.
- `[x]` Implement `Easing::Linear`.
- `[x]` Implement `Easing::EaseIn`.
- `[x]` Implement `Easing::EaseOut`.
- `[x]` Implement `Easing::EaseInOut`.
- `[x]` Implement `Easing::EaseInCubic`.
- `[x]` Implement `Easing::EaseOutCubic`.
- `[x]` Implement `Easing::EaseInOutCubic`.
- `[x]` Implement Sine easing curves.
- `[x]` Implement Circ easing curves.
- `[x]` Implement Expo easing curves.
- `[x]` Implement cubic bezier easing support.
- `[x]` Implement material-style standard easing curves.
- `[x]` Add Rustdoc examples for built-in easing curves.
- `[ ]` Add Rustdoc performance notes for easing sampling.
- `[ ]` Export `Easing` from `src/lib.rs`.

### `easing` Tests

- `[ ]` Test `Easing::Linear` at progress `0.0`.
- `[ ]` Test `Easing::Linear` at progress `0.5`.
- `[ ]` Test `Easing::Linear` at progress `1.0`.
- `[ ]` Test progress clamping below `0.0`.
- `[ ]` Test progress clamping above `1.0`.
- `[ ]` Test cubic easing endpoints.
- `[ ]` Test cubic bezier endpoints.
- `[ ]` Test finite output for all built-in curves.

## `interpolate`

- `[ ]` Create the public `Interpolate` trait in `src/interpolate.rs`.
- `[ ]` Implement `Interpolate` for `f32`.
- `[ ]` Implement `Interpolate` for `f64`.
- `[ ]` Implement `Interpolate` for `u8`.
- `[ ]` Implement `Interpolate` for `i32`.
- `[ ]` Implement `Interpolate` for `(T, T)`.
- `[ ]` Implement `Interpolate` for `(T, T, T)`.
- `[ ]` Implement `Interpolate` for `(T, T, T, T)`.
- `[ ]` Implement progress clamping for interpolation.
- `[ ]` Implement documented integer rounding behavior.
- `[ ]` Add Rustdoc examples for custom `Interpolate` implementations.
- `[ ]` Export `Interpolate` from `src/lib.rs`.

### `interpolate` Tests

- `[ ]` Test `f32` interpolation at progress `0.0`.
- `[ ]` Test `f32` interpolation at progress `0.5`.
- `[ ]` Test `f32` interpolation at progress `1.0`.
- `[ ]` Test `f64` interpolation at progress `0.5`.
- `[ ]` Test `u8` interpolation rounding.
- `[ ]` Test `i32` interpolation rounding.
- `[ ]` Test tuple interpolation at progress `0.5`.
- `[ ]` Test interpolation progress clamping below `0.0`.
- `[ ]` Test interpolation progress clamping above `1.0`.

## `value`

- `[ ]` Implement Iced `Color` interpolation in `src/value/color.rs`.
- `[ ]` Write RGB color interpolation documentation in `src/value/color.rs`.
- `[ ]` Implement opacity interpolation helper in `src/value/style.rs`.
- `[ ]` Implement radius interpolation helper in `src/value/style.rs`.
- `[ ]` Implement size interpolation helper in `src/value/geometry.rs`.
- `[ ]` Implement point interpolation helper in `src/value/geometry.rs`.
- `[ ]` Implement vector interpolation helper in `src/value/geometry.rs`.
- `[ ]` Export value helpers from `src/value/mod.rs`.
- `[ ]` Add Rustdoc examples for color interpolation.
- `[ ]` Add Rustdoc examples for geometry interpolation.

### `value` Tests

- `[ ]` Test Iced `Color` interpolation at progress `0.0`.
- `[ ]` Test Iced `Color` interpolation at progress `0.5`.
- `[ ]` Test Iced `Color` interpolation at progress `1.0`.
- `[ ]` Test opacity interpolation endpoints.
- `[ ]` Test radius interpolation endpoints.
- `[ ]` Test size interpolation endpoints.
- `[ ]` Test point interpolation endpoints.
- `[ ]` Test vector interpolation endpoints.

## `tween`

- `[ ]` Create the public `Tween<T>` type in `src/tween.rs`.
- `[ ]` Implement `Tween::new`.
- `[ ]` Implement duration configuration for `Tween<T>`.
- `[ ]` Implement delay configuration for `Tween<T>`.
- `[ ]` Implement easing configuration for `Tween<T>`.
- `[ ]` Implement `Tween::sample`.
- `[ ]` Implement `Tween::progress`.
- `[ ]` Implement `Tween::is_complete`.
- `[ ]` Implement zero-duration sampling behavior.
- `[ ]` Implement delayed sampling behavior.
- `[ ]` Implement elapsed-over-duration sampling behavior.
- `[ ]` Add Rustdoc examples for numeric tween usage.
- `[ ]` Add Rustdoc examples for delayed tween usage.
- `[ ]` Export `Tween` from `src/lib.rs`.

### `tween` Tests

- `[ ]` Test `Tween::new` initial sampling.
- `[ ]` Test sampling before delay.
- `[ ]` Test sampling at delay boundary.
- `[ ]` Test sampling at half duration.
- `[ ]` Test sampling at full duration.
- `[ ]` Test sampling after full duration.
- `[ ]` Test zero-duration sampling.
- `[ ]` Test `Tween::progress` before delay.
- `[ ]` Test `Tween::progress` at half duration.
- `[ ]` Test `Tween::progress` after full duration.
- `[ ]` Test `Tween::is_complete` before completion.
- `[ ]` Test `Tween::is_complete` after completion.
- `[ ]` Test easing application in `Tween::sample`.

## `transition`

- `[ ]` Create the public `Transition<T>` type in `src/transition.rs`.
- `[ ]` Implement `Transition::new`.
- `[ ]` Implement duration configuration for `Transition<T>`.
- `[ ]` Implement easing configuration for `Transition<T>`.
- `[ ]` Implement `Transition::set_target`.
- `[ ]` Implement `Transition::value_at`.
- `[ ]` Implement `Transition::tick`.
- `[ ]` Implement `Transition::is_animating`.
- `[ ]` Implement `Transition::current`.
- `[ ]` Implement `Transition::target`.
- `[ ]` Implement active retargeting from the sampled current value.
- `[ ]` Implement same-target handling.
- `[ ]` Implement zero-duration transition behavior.
- `[ ]` Add Rustdoc examples for visibility transitions.
- `[ ]` Add Rustdoc examples for hover transitions.
- `[ ]` Add Rustdoc examples for expanded-state transitions.
- `[ ]` Export `Transition` from `src/lib.rs`.

### `transition` Tests

- `[ ]` Test inactive state after `Transition::new`.
- `[ ]` Test active state after `Transition::set_target`.
- `[ ]` Test same-target handling.
- `[ ]` Test midpoint value with linear easing.
- `[ ]` Test completion state after duration.
- `[ ]` Test active retargeting from the sampled current value.
- `[ ]` Test zero-duration transition.
- `[ ]` Test repeated `value_at` calls for the same timestamp.
- `[ ]` Test `current` after `tick`.
- `[ ]` Test `target` after `set_target`.

## `clock`

- `[ ]` Create the public animation clock type in `src/clock.rs`.
- `[ ]` Implement fixed 16 ms tick interval support.
- `[ ]` Implement active animation count tracking.
- `[ ]` Implement clock activation.
- `[ ]` Implement clock deactivation.
- `[ ]` Implement deterministic time advancement under the `testing` feature.
- `[ ]` Add Rustdoc examples for clock usage.
- `[ ]` Export clock types from `src/lib.rs`.

### `clock` Tests

- `[ ]` Test fixed tick interval creation.
- `[ ]` Test active animation count increment.
- `[ ]` Test active animation count decrement.
- `[ ]` Test clock activation.
- `[ ]` Test clock deactivation.
- `[ ]` Test deterministic time advancement under the `testing` feature.

## Iced Integration

- `[ ]` Implement the Iced subscription helper in `src/iced_ext.rs`.
- `[ ]` Implement inactive subscription behavior.
- `[ ]` Implement active subscription behavior with a 16 ms interval.
- `[ ]` Implement message mapping examples in Rustdoc.
- `[ ]` Gate Iced integration with the `iced` feature.
- `[ ]` Export Iced helpers from `src/lib.rs`.

### Iced Integration Tests

- `[ ]` Test Iced helper compilation with default features.
- `[ ]` Test Iced helper compilation with `--all-features`.
- `[ ]` Test core crate compilation with `--no-default-features`.
- `[ ]` Test subscription helper type signatures in an integration test.

## Examples

- `[ ]` Create `examples/basic_tween.rs`.
- `[ ]` Implement a numeric tween demo in `examples/basic_tween.rs`.
- `[ ]` Create `examples/fade_panel.rs`.
- `[ ]` Implement an opacity transition demo in `examples/fade_panel.rs`.
- `[ ]` Create `examples/slide_panel.rs`.
- `[ ]` Implement an offset transition demo in `examples/slide_panel.rs`.
- `[ ]` Add README commands for running `basic_tween`.
- `[ ]` Add README commands for running `fade_panel`.
- `[ ]` Add README commands for running `slide_panel`.
- `[ ]` Test all examples with `cargo check --examples`.

## Tests

- `[ ]` Create integration test directory `tests/`.
- `[ ]` Create `tests/public_api.rs`.
- `[ ]` Write public API smoke tests in `tests/public_api.rs`.
- `[ ]` Create `tests/feature_matrix.rs`.
- `[ ]` Add module-level unit tests for `easing`.
- `[ ]` Add module-level unit tests for `interpolate`.
- `[ ]` Add module-level unit tests for `tween`.
- `[ ]` Add module-level unit tests for `transition`.
- `[ ]` Add module-level unit tests for `clock`.
- `[ ]` Add module-level unit tests for `value`.
- `[ ]` Run `cargo test --all-features`.
- `[ ]` Run `cargo check --all-targets`.
- `[ ]` Run `cargo check --all-targets --all-features`.
- `[ ]` Run `cargo check --no-default-features`.
- `[ ]` Run `cargo check --examples`.

## Benchmarks

- `[ ]` Add benchmark harness configuration.
- `[ ]` Create `benches/tween.rs`.
- `[ ]` Benchmark single `Tween::sample` calls.
- `[ ]` Create `benches/transition.rs`.
- `[ ]` Benchmark single `Transition::value_at` calls.
- `[ ]` Benchmark 1000 transition value queries.
- `[ ]` Create `benches/easing.rs`.
- `[ ]` Benchmark built-in easing sampling.
- `[ ]` Record benchmark command output in `CHANGELOG.md`.
- `[ ]` Add benchmark instructions to README.

## Documentation

- `[ ]` Expand README project positioning.
- `[ ]` Add README installation instructions.
- `[ ]` Add README minimal tween example.
- `[ ]` Add README transition example.
- `[ ]` Add README Iced integration example.
- `[ ]` Add README feature flag table.
- `[ ]` Add README examples section.
- `[ ]` Add README supported Iced version.
- `[ ]` Add README MSRV.
- `[ ]` Add README crates.io release instructions.
- `[ ]` Add crate-level Rustdoc overview in `src/lib.rs`.
- `[ ]` Add public item Rustdoc for `Easing`.
- `[ ]` Add public item Rustdoc for `Interpolate`.
- `[ ]` Add public item Rustdoc for `Tween`.
- `[ ]` Add public item Rustdoc for `Transition`.
- `[ ]` Add public item Rustdoc for clock helpers.
- `[ ]` Add public item Rustdoc for Iced helpers.
- `[ ]` Add public item Rustdoc for value helpers.

## Feature Flags

- `[ ]` Test default features with `cargo check --all-targets`.
- `[ ]` Test all features with `cargo check --all-targets --all-features`.
- `[ ]` Test `serde` with `cargo check --features serde`.
- `[ ]` Test `tracing` with `cargo check --features tracing`.
- `[ ]` Test `testing` with `cargo check --features testing`.
- `[ ]` Test `widgets` with `cargo check --features widgets`.
- `[ ]` Test `spring` with `cargo check --features spring`.
- `[ ]` Test core features with `cargo check --no-default-features`.
- `[ ]` Add CI entries for required feature combinations.

## CI

- `[ ]` Add `cargo check --examples` to `.github/workflows/ci.yml`.
- `[ ]` Add `cargo check --no-default-features` to `.github/workflows/ci.yml`.
- `[ ]` Add feature-specific check commands to `.github/workflows/ci.yml`.
- `[ ]` Run `cargo fmt --all --check`.
- `[ ]` Run `cargo clippy --all-targets --all-features -- -D warnings`.
- `[ ]` Run `cargo doc --no-deps --all-features`.
- `[ ]` Run `cargo package --all-features`.
- `[ ]` Fix every CI failure.

## Release Preparation

- `[ ]` Update `CHANGELOG.md` with v0.1.0 changes.
- `[ ]` Update package metadata in `Cargo.toml`.
- `[ ]` Run `cargo package --all-features`.
- `[ ]` Inspect package files with `cargo package --list`.
- `[ ]` Run `cargo publish --dry-run --all-features`.
- `[ ]` Tag the release with `v0.1.0`.
- `[ ]` Push the release tag.
- `[ ]` Publish the crate to crates.io.

## v0.1 Release Gate

- `[ ]` Run `cargo fmt --all --check`.
- `[ ]` Run `cargo check --all-targets`.
- `[ ]` Run `cargo check --all-targets --all-features`.
- `[ ]` Run `cargo check --no-default-features`.
- `[ ]` Run `cargo check --examples`.
- `[ ]` Run `cargo test --all-features`.
- `[ ]` Run `cargo clippy --all-targets --all-features -- -D warnings`.
- `[ ]` Run `cargo doc --no-deps --all-features`.
- `[ ]` Run `cargo package --all-features`.
- `[ ]` Run `cargo publish --dry-run --all-features`.
- `[ ]` Run `examples/basic_tween.rs`.
- `[ ]` Run `examples/fade_panel.rs`.
- `[ ]` Run `examples/slide_panel.rs`.
- `[ ]` Record manual example verification in `CHANGELOG.md`.
