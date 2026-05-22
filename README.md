# aura-anim-iced

`aura-anim-iced` is an Iced-first Rust animation crate.

The crate is intended to provide reusable UI animation primitives for Iced
applications: easing, interpolation, tweening, state transitions, timelines,
animation clocks, and optional widget helpers.

## Scope

- Iced application and widget animation support.
- Explicit animation state owned by the application, component, or widget.
- Deterministic animation calculations that can be tested without a GUI.
- Low-overhead integration with Iced `Subscription`-driven ticks.

## Features

| Feature | Default | Purpose |
| --- | --- | --- |
| `iced` | yes | Iced integration helpers. |
| `serde` | no | Serialization support for future configuration types. |
| `tracing` | no | Diagnostic instrumentation. |
| `testing` | no | Deterministic test helpers. |
| `widgets` | no | Optional animated widget helpers. |
| `spring` | no | Reserved for future spring animation support. |

## Development

```sh
cargo fmt --all --check
cargo check --all-targets --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
```

The current repository is a scaffold. Public animation APIs will be added as
the v0.1 milestones are implemented.
